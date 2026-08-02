// SPDX-License-Identifier: MIT

use super::*;

pub(super) fn resource_url_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r#"(?i)(?P<url>(?P<id>[\p{L}0-9_-]+\.[\p{L}0-9_.-]*[\p{L}0-9_-]\.(?:latest|min\d+|\d+))(?:\.var)?:(?:/|\\)[^"'\r\n]+)"#,
        )
        .unwrap()
    })
}

pub(super) fn resource_urls_in_text(text: &str) -> BTreeSet<String> {
    resource_url_regex()
        .captures_iter(text)
        .filter_map(|capture| {
            parse_reference(capture.name("id")?.as_str())?;
            Some(capture.name("url")?.as_str().to_string())
        })
        .collect()
}

fn resolve_reference_package<'a>(
    reference: &PackageRef,
    index: &'a HashMap<String, Vec<&'a VarPackage>>,
) -> Option<&'a VarPackage> {
    let resolved = resolve_reference(reference, index)?;
    index
        .get(&reference.family())?
        .iter()
        .copied()
        .find(|package| {
            package
                .id
                .as_ref()
                .is_some_and(|id| id.display() == resolved)
        })
}

pub(super) fn missing_resource_members(
    packages: &[VarPackage],
    index: &HashMap<String, Vec<&VarPackage>>,
) -> Vec<String> {
    let mut missing = BTreeSet::new();
    for owner in packages.iter().filter(|package| package.valid) {
        for raw in &owner.resource_urls {
            let Some((package_part, member_part)) = raw.split_once(':') else {
                continue;
            };
            let Some(reference) = parse_reference(package_part) else {
                continue;
            };
            let Some(provider) = resolve_reference_package(&reference, index) else {
                continue;
            };
            let member = member_part
                .trim_start_matches(['/', '\\'])
                .replace('\\', "/");
            if provider.entries.contains(&member) {
                continue;
            }
            let case_matches = provider
                .entries
                .iter()
                .filter(|entry| entry.eq_ignore_ascii_case(&member))
                .count();
            let detail = if case_matches == 1 {
                "the member exists with different path casing; the source reference needs a casing repair"
            } else {
                "the installed provider does not contain this member; reacquire a known-good provider before editing the source"
            };
            missing.insert(format!(
                "MISSING MEMBER: {} :: {} -> {} :: {detail}",
                owner.relative.display(),
                raw,
                provider.relative.display()
            ));
        }
    }
    missing.into_iter().collect()
}

fn diagnose_logged_resource_member(raw: &str, index: &HashMap<String, Vec<&VarPackage>>) -> String {
    let Some((package_part, member_part)) = raw.split_once(':') else {
        return format!("VAM-CONFIRMED MISSING MEMBER: {raw}");
    };
    let Some(reference) = parse_reference(package_part) else {
        return format!("VAM-CONFIRMED MISSING MEMBER: {raw}");
    };
    let Some(provider) = resolve_reference_package(&reference, index) else {
        return format!(
            "VAM-CONFIRMED MISSING MEMBER: {raw} :: no installed provider resolves this package ID"
        );
    };
    let member = member_part
        .trim_start_matches(['/', '\\'])
        .replace('\\', "/");
    let case_matches = provider
        .entries
        .iter()
        .filter(|entry| entry.eq_ignore_ascii_case(&member))
        .count();
    let detail = if case_matches == 1 {
        "the member exists with different path casing; repair the source reference casing"
    } else {
        "the installed provider does not contain this member; reacquire a known-good provider"
    };
    format!(
        "VAM-CONFIRMED MISSING MEMBER: {raw} -> {} :: {detail}",
        provider.relative.display()
    )
}

pub(super) fn logged_missing_resource_members(
    log: &VamLogData,
    index: &HashMap<String, Vec<&VarPackage>>,
) -> Vec<String> {
    log.missing_resource_urls
        .iter()
        .map(|raw| diagnose_logged_resource_member(raw, index))
        .collect()
}

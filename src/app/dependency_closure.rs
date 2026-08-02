// SPDX-License-Identifier: MIT

use super::*;

#[derive(Default)]
pub(super) struct DependencyRelinkResult {
    pub(super) taken: Vec<String>,
    pub(super) required: Vec<String>,
    pub(super) relinked: usize,
    pub(super) metadata_cleaned: usize,
    pub(super) failed: bool,
}

pub(super) fn build_dependency_relink_plan(
    packages: &[VarPackage],
    index: &HashMap<String, Vec<&VarPackage>>,
    vam_log: Option<&VamLogData>,
) -> BTreeMap<usize, HashMap<String, String>> {
    let mut plan = BTreeMap::new();
    for (position, package) in packages.iter().enumerate() {
        if !package.valid || package.is_plugin {
            continue;
        }
        let replacements = package_dependencies(package, vam_log)
            .into_iter()
            .filter_map(|raw| {
                let reference = parse_reference(&raw)?;
                let resolved = resolve_reference(&reference, index)?;
                let is_local =
                    package.content_refs.contains(&raw) || package.declared_refs.contains_key(&raw);
                (is_local && resolved != raw && is_safe_non_plugin_relink(&reference, &resolved))
                    .then_some((raw, resolved))
            })
            .collect::<HashMap<_, _>>();
        let needs_metadata_cleanup = replacements.is_empty()
            && repaired_metadata(package, None, false)
                .ok()
                .flatten()
                .is_some_and(|metadata| package.meta.as_ref() != Some(&metadata));
        if !replacements.is_empty() || needs_metadata_cleanup {
            plan.insert(position, replacements);
        }
    }
    plan
}

fn dependency_relink_metadata(
    package: &VarPackage,
    replacements: &HashMap<String, String>,
) -> Result<Option<Value>> {
    let Some(base) = repaired_metadata(package, None, false)? else {
        return Ok(None);
    };
    match migrate_metadata(&base, replacements)? {
        Some(updated) => Ok(Some(updated)),
        None if package.meta.as_ref() != Some(&base) => Ok(Some(base)),
        None => Ok(None),
    }
}

pub(super) fn apply_dependency_relink_plan(
    packages: &[VarPackage],
    plan: &BTreeMap<usize, HashMap<String, String>>,
    root: &Path,
    backup: &Path,
) -> DependencyRelinkResult {
    let mut result = DependencyRelinkResult::default();
    for (position, replacements) in plan {
        let package = &packages[*position];
        let needs_content = package
            .content_refs
            .iter()
            .any(|reference| replacements.contains_key(reference));
        let metadata = match dependency_relink_metadata(package, replacements) {
            Ok(metadata) => metadata,
            Err(error) => {
                result.required.push(format!(
                    "SKIP RELINK {} :: cannot safely update metadata: {error}",
                    package.relative.display()
                ));
                result.failed = true;
                continue;
            }
        };
        if metadata.is_none() && !needs_content {
            continue;
        }
        if let Err(error) = atomic_rewrite(
            package,
            metadata.as_ref(),
            replacements,
            root,
            backup,
            "dependency-relink",
        ) {
            result.required.push(format!(
                "FAILED RELINK {} :: {error}",
                package.relative.display()
            ));
            result.failed = true;
            break;
        }
        if replacements.is_empty() {
            result.metadata_cleaned += 1;
            result
                .taken
                .push(format!("METADATA CLEANUP: {}", package.relative.display()));
        } else {
            result.relinked += 1;
            result.taken.push(format!(
                "RELINKED: {} :: {} reference(s)",
                package.relative.display(),
                replacements.len()
            ));
        }
    }
    result
}

fn seed_dependency_failure_reasons(
    packages: &[VarPackage],
    vam_log: Option<&VamLogData>,
) -> Vec<BTreeSet<String>> {
    packages
        .iter()
        .map(|package| {
            let mut reasons = BTreeSet::new();
            if !package.valid {
                reasons.insert(format!("invalid archive: {}", package.issues.join("; ")));
            }
            if let (Some(id), Some(log)) = (&package.id, vam_log)
                && log.corrupt_packages.contains(&package_key(id))
                && !log.header_mismatch_packages.contains(&package_key(id))
            {
                reasons.insert("VaM rejected this archive while loading meta.json".to_string());
            }
            reasons
        })
        .collect()
}

pub(super) fn dependency_failure_closure(
    packages: &[VarPackage],
    index: &HashMap<String, Vec<&VarPackage>>,
    vam_log: Option<&VamLogData>,
) -> (Vec<BTreeSet<String>>, BTreeSet<String>) {
    let mut reasons = seed_dependency_failure_reasons(packages, vam_log);
    let mut triggering_missing = BTreeSet::new();
    loop {
        let removed_ids = packages
            .iter()
            .enumerate()
            .filter(|(position, package)| !reasons[*position].is_empty() && package.id.is_some())
            .map(|(_, package)| package_key(package.id.as_ref().expect("filtered package ID")))
            .collect::<HashSet<_>>();
        let changed = expand_dependency_failure_closure(
            packages,
            index,
            vam_log,
            &removed_ids,
            &mut reasons,
            &mut triggering_missing,
        );
        if !changed {
            return (reasons, triggering_missing);
        }
    }
}

fn expand_dependency_failure_closure(
    packages: &[VarPackage],
    index: &HashMap<String, Vec<&VarPackage>>,
    vam_log: Option<&VamLogData>,
    removed_ids: &HashSet<String>,
    reasons: &mut [BTreeSet<String>],
    triggering_missing: &mut BTreeSet<String>,
) -> bool {
    let mut changed = false;
    for (position, package) in packages.iter().enumerate() {
        if !package.valid || package.id.is_none() || !reasons[position].is_empty() {
            continue;
        }
        for raw in package_dependencies(package, vam_log) {
            let Some(reference) = parse_reference(&raw) else {
                if dependency_is_missing_in_log(package, &raw, vam_log) {
                    triggering_missing.insert(raw.clone());
                    reasons[position]
                        .insert(format!("VaM reports unparseable missing dependency {raw}"));
                    changed = true;
                }
                continue;
            };
            let provider =
                resolve_reference_excluding(&reference, index, removed_ids, package.is_plugin);
            if provider.is_none() && !can_drop_empty_metadata_dependency(package, &raw) {
                triggering_missing.insert(raw.clone());
                reasons[position].insert(format!(
                    "unresolved {} dependency {raw}",
                    if package.is_plugin {
                        "strict script"
                    } else {
                        "package"
                    }
                ));
                changed = true;
            }
        }
    }
    changed
}

fn dependency_is_missing_in_log(
    package: &VarPackage,
    dependency: &str,
    vam_log: Option<&VamLogData>,
) -> bool {
    let Some(id) = package.id.as_ref() else {
        return false;
    };
    vam_log.is_some_and(|log| {
        log.missing_by_owner
            .get(&package_key(id))
            .is_some_and(|values| values.contains(dependency))
    })
}

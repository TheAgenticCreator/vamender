// SPDX-License-Identifier: MIT

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::NamedTempFile;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

const TEXT_LIMIT: u64 = 16 * 1024 * 1024;
// VaM libraries sometimes contain archives whose decompressor or metadata is
// much deeper than a normal Windows main-thread stack. Keep the main thread for
// argument parsing only, run commands on a large dedicated stack, and cap the
// number of equally large scan workers so a large library does not reserve one
// giant stack per CPU core.
const COMMAND_STACK_BYTES: usize = 256 * 1024 * 1024;
const SCAN_COORDINATOR_STACK_BYTES: usize = 128 * 1024 * 1024;
const SCAN_WORKER_STACK_BYTES: usize = 128 * 1024 * 1024;
const MAX_SCAN_THREADS: usize = 4;
const LICENSES: &[&str] = &[
    "CC0",
    "CC BY",
    "CC BY-SA",
    "CC BY-NC",
    "CC BY-NC-SA",
    "CC BY-NC-ND",
    "CC BY-ND",
    "FC",
    "PC",
    "PC EA",
    "Other",
];

mod bridge;
mod cli;
mod dependency_closure;
mod filename_repair;
mod host_install;
mod model;
mod resource_members;
mod support_report;
mod tray_host;

use bridge::*;
use cli::*;
use dependency_closure::*;
use filename_repair::*;
use host_install::*;
use model::*;
use resource_members::*;
use support_report::*;
use tray_host::*;

fn text_extensions() -> &'static [&'static str] {
    &[
        "json", "vap", "vaj", "cslist", "txt", "xml", "html", "htm", "cs",
    ]
}

fn package_file_id(name: &str) -> Option<PackageId> {
    let base = name
        .strip_suffix(".var")
        .or_else(|| name.strip_suffix(".VAR"))?;
    let (prefix, version) = base.rsplit_once('.')?;
    let version = version.parse().ok()?;
    let (creator, package) = prefix.split_once('.')?;
    if creator.is_empty() || package.is_empty() || package.contains('.') {
        return None;
    }
    Some(PackageId {
        creator: creator.to_string(),
        package: package.to_string(),
        version,
    })
}

fn parse_reference(value: &str) -> Option<PackageRef> {
    let clean = value.trim().strip_suffix(".var").unwrap_or(value.trim());
    let (prefix, selector) = clean.rsplit_once('.')?;
    if !(selector == "latest" || selector.starts_with("min") || selector.parse::<u32>().is_ok()) {
        return None;
    }
    let (creator, package) = prefix.split_once('.')?;
    if creator.trim().is_empty() || package.trim().is_empty() {
        return None;
    }
    // Reject decimal/morph/material strings that can coincidentally look like
    // three dot-separated tokens. Real creator and package names contain at
    // least one letter; Unicode creator/package names are valid in VaM, while
    // creators such as 14mhz still satisfy this rule.
    if !creator.chars().any(|character| character.is_alphabetic())
        || !package.chars().any(|character| character.is_alphabetic())
    {
        return None;
    }
    if creator.chars().all(|c| c.is_ascii_digit()) && package.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(PackageRef {
        raw: clean.to_string(),
        creator: creator.to_string(),
        package: package.to_string(),
        selector: selector.to_string(),
    })
}

fn reference_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?i)(?P<id>[\p{L}0-9_-]+\.[\p{L}0-9_.-]*[\p{L}0-9_-]\.(?:latest|min\d+|\d+))(?:\.var)?").unwrap())
}

fn resource_reference_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r"(?i)(?P<id>[\p{L}0-9_-]+\.[\p{L}0-9_.-]*[\p{L}0-9_-]\.(?:latest|min\d+|\d+))(?:\.var)?:(?:/|\\)",
        )
        .unwrap()
    })
}

fn resource_references_in_text(text: &str) -> BTreeSet<String> {
    resource_reference_regex()
        .captures_iter(text)
        .filter_map(|capture| parse_reference(capture.name("id")?.as_str()).map(|item| item.raw))
        .collect()
}

fn is_dependency_field(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "dependency" | "dependencies" | "package" | "packageid" | "var" | "varname" | "varpackage"
    )
}

fn collect_json_references(value: &Value, field_name: Option<&str>, output: &mut BTreeSet<String>) {
    match value {
        Value::Object(object) => {
            for (name, child) in object {
                collect_json_references(child, Some(name), output);
            }
        }
        Value::Array(values) => {
            for child in values {
                collect_json_references(child, field_name, output);
            }
        }
        Value::String(value) => {
            output.extend(resource_references_in_text(value));
            if field_name.is_some_and(is_dependency_field)
                && let Some(reference) = parse_reference(value)
            {
                output.insert(reference.raw);
            }
        }
        _ => {}
    }
}

// Only resource URLs (`Creator.Package.Version:/...`) are unambiguous in free
// text. JSON can additionally use an explicit dependency/package field. The
// old broad dotted-token regex turned material labels such as
// `base.spec.1001` into fake package requirements.
fn references_in_text(text: &str) -> BTreeSet<String> {
    let mut output = resource_references_in_text(text);
    if let Ok(value) = serde_json::from_str::<Value>(text) {
        collect_json_references(&value, None, &mut output);
    }
    output
}

fn is_text_member(name: &str) -> bool {
    Path::new(name)
        .extension()
        .and_then(|v| v.to_str())
        .map(|v| {
            text_extensions()
                .iter()
                .any(|extension| v.eq_ignore_ascii_case(extension))
        })
        .unwrap_or(false)
}

fn read_text(mut member: zip::read::ZipFile<'_>) -> Option<String> {
    if member.size() > TEXT_LIMIT {
        return None;
    }
    let mut raw = Vec::with_capacity(member.size() as usize);
    member.read_to_end(&mut raw).ok()?;
    if raw.starts_with(&[0xff, 0xfe]) || raw.starts_with(&[0xfe, 0xff]) {
        return None;
    }
    String::from_utf8(raw).ok()
}

fn add_metadata_from_text(result: &mut VarPackage, text: &str) {
    match serde_json::from_str::<Value>(text) {
        Ok(meta) if meta.is_object() => {
            if let Some(dependencies) = meta.get("dependencies").and_then(Value::as_object) {
                for (name, payload) in dependencies {
                    result.declared_refs.insert(name.clone(), payload.clone());
                }
            }
            result.meta = Some(meta);
        }
        _ => result.issues.push("invalid meta.json".to_string()),
    }
}

fn finalize_package_validity(result: &mut VarPackage) {
    // Metadata can be rebuilt. Only an archive/path/CRC failure makes a VAR
    // ineligible for repair or for providing a locally installed version.
    result.valid = !result
        .issues
        .iter()
        .any(|issue| !(issue == "missing meta.json" || issue == "invalid meta.json"));
}

// Some older but VaM-readable VARs use ZIP layouts that the primary ZIP reader
// rejects before it can find the EOCD. Windows' bundled tar (and VaM itself)
// can read those archives. Use it only as a compatibility fallback: it proves
// the archive can be enumerated and lets the package provide its declared
// metadata, but does not claim that a deep CRC pass was performed.
#[cfg(windows)]
fn populate_tar_compatibility_fallback(path: &Path, result: &mut VarPackage) -> bool {
    let listing = match std::process::Command::new("tar.exe")
        .arg("-tf")
        .arg(path)
        .output()
    {
        Ok(output) if output.status.success() => output,
        _ => return false,
    };
    let listing = match String::from_utf8(listing.stdout) {
        Ok(listing) => listing,
        Err(_) => return false,
    };
    for raw_name in listing.lines() {
        let name = raw_name.trim().replace('\\', "/");
        if name.is_empty() {
            continue;
        }
        if name.starts_with('/') || name.split('/').any(|part| part == "..") {
            result.issues.push(format!("unsafe member path: {name}"));
        }
        result.is_plugin |= name.to_ascii_lowercase().starts_with("custom/scripts/");
        result.entries.insert(name);
    }
    let metas: Vec<_> = result
        .entries
        .iter()
        .filter(|name| name.eq_ignore_ascii_case("meta.json"))
        .cloned()
        .collect();
    if metas.len() == 1 {
        result.meta_name = metas.first().cloned();
        let output = std::process::Command::new("tar.exe")
            .arg("-xOf")
            .arg(path)
            .arg(metas.first().unwrap())
            .output();
        match output {
            Ok(output) if output.status.success() => match String::from_utf8(output.stdout) {
                Ok(text) => add_metadata_from_text(result, &text),
                Err(_) => result.issues.push("cannot decode meta.json".to_string()),
            },
            _ => result.issues.push("cannot read meta.json".to_string()),
        }
    } else if metas.is_empty() {
        result.issues.push("missing meta.json".to_string());
    } else {
        result
            .issues
            .push("multiple root meta.json files".to_string());
    }
    finalize_package_validity(result);
    true
}

fn scan_package(path: &Path, root: &Path, deep: bool) -> VarPackage {
    let relative = path.strip_prefix(root).unwrap_or(path).to_path_buf();
    let id = path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(package_file_id);
    let mut result = VarPackage {
        path: path.to_path_buf(),
        relative,
        id,
        valid: false,
        issues: Vec::new(),
        entries: BTreeSet::new(),
        is_plugin: false,
        meta_name: None,
        meta: None,
        content_refs: BTreeSet::new(),
        resource_urls: BTreeSet::new(),
        declared_refs: BTreeMap::new(),
    };
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) => {
            result.issues.push(format!("open failed: {error}"));
            return result;
        }
    };
    let mut archive = match ZipArchive::new(file) {
        Ok(archive) => archive,
        Err(error) => {
            #[cfg(windows)]
            if populate_tar_compatibility_fallback(path, &mut result) {
                return result;
            }
            result.issues.push(format!("invalid ZIP: {error}"));
            return result;
        }
    };
    for index in 0..archive.len() {
        match archive.by_index(index) {
            Ok(member) => {
                let name = member.name().replace('\\', "/");
                if name.starts_with('/') || name.split('/').any(|part| part == "..") {
                    result.issues.push(format!("unsafe member path: {name}"));
                }
                result.is_plugin |= name.to_ascii_lowercase().starts_with("custom/scripts/");
                result.entries.insert(name);
            }
            Err(error) => result
                .issues
                .push(format!("cannot inspect member: {error}")),
        }
    }
    let metas: Vec<_> = result
        .entries
        .iter()
        .filter(|name| name.eq_ignore_ascii_case("meta.json"))
        .cloned()
        .collect();
    if metas.len() == 1 {
        result.meta_name = metas.first().cloned();
        if let Ok(member) = archive.by_name(metas.first().unwrap()) {
            if let Some(text) = read_text(member) {
                add_metadata_from_text(&mut result, &text);
            } else {
                result.issues.push("cannot decode meta.json".to_string());
            }
        }
    } else if metas.is_empty() {
        result.issues.push("missing meta.json".to_string());
    } else {
        result
            .issues
            .push("multiple root meta.json files".to_string());
    }

    let entry_names: Vec<_> = result.entries.iter().cloned().collect();
    for name in entry_names {
        if !is_text_member(&name) || name.eq_ignore_ascii_case("meta.json") {
            continue;
        }
        match archive.by_name(&name) {
            Ok(member) => {
                if let Some(text) = read_text(member) {
                    result.content_refs.extend(references_in_text(&text));
                    result.resource_urls.extend(resource_urls_in_text(&text));
                }
            }
            Err(error) => result.issues.push(format!("cannot read {name}: {error}")),
        }
    }
    if deep {
        for index in 0..archive.len() {
            let mut buffer = [0u8; 64 * 1024];
            match archive.by_index(index) {
                Ok(mut member) => loop {
                    match member.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(_) => {}
                        Err(error) => {
                            result
                                .issues
                                .push(format!("CRC/read error in {}: {error}", member.name()));
                            break;
                        }
                    }
                },
                Err(error) => result
                    .issues
                    .push(format!("cannot read archive member: {error}")),
            }
        }
    }
    finalize_package_validity(&mut result);
    result
}

fn scan_library(root: &Path, deep: bool) -> Result<Vec<VarPackage>> {
    if !root.is_dir() {
        bail!("AddonPackages folder does not exist: {}", root.display());
    }
    let mut paths: Vec<_> = WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .and_then(|value| value.to_str())
                    .is_some_and(|value| value.eq_ignore_ascii_case("var"))
        })
        .map(|entry| entry.into_path())
        .collect();
    paths.sort();
    let root = root.to_path_buf();
    let scan_threads = std::thread::available_parallelism()
        .map(|value| value.get())
        .unwrap_or(1)
        .clamp(1, MAX_SCAN_THREADS);
    // A few unusually nested/compressed community archives can exceed the
    // default Windows main-thread stack. Isolate each scan in a capped pool of
    // large-stack workers: this is safer than creating one large worker per
    // logical CPU when a library contains thousands of VARs.
    std::thread::Builder::new()
        .name("vam-var-scan-coordinator".to_string())
        .stack_size(SCAN_COORDINATOR_STACK_BYTES)
        .spawn(move || {
            rayon::ThreadPoolBuilder::new()
                .num_threads(scan_threads)
                .stack_size(SCAN_WORKER_STACK_BYTES)
                .build()
                .context("cannot create parallel scan pool")?
                .install(|| {
                    Ok(paths
                        .par_iter()
                        .map(|path| scan_package(path, &root, deep))
                        .collect())
                })
        })
        .context("cannot start large-stack scan worker")?
        .join()
        .map_err(|_| anyhow::anyhow!("VAR scan worker panicked"))?
}

fn providers(packages: &[VarPackage]) -> HashMap<String, Vec<&VarPackage>> {
    let mut index: HashMap<String, Vec<&VarPackage>> = HashMap::new();
    for package in packages.iter().filter(|item| item.valid) {
        if let Some(id) = &package.id {
            index.entry(id.family()).or_default().push(package);
        }
    }
    for versions in index.values_mut() {
        versions.sort_by_key(|item| item.id.as_ref().unwrap().version);
    }
    index
}

fn resolve_reference(
    reference: &PackageRef,
    index: &HashMap<String, Vec<&VarPackage>>,
) -> Option<String> {
    let options = index.get(&reference.family())?;
    if reference.selector == "latest" {
        return options
            .last()
            .and_then(|item| item.id.as_ref().map(PackageId::display));
    }
    if let Some(minimum) = reference
        .selector
        .strip_prefix("min")
        .and_then(|value| value.parse::<u32>().ok())
    {
        return options
            .iter()
            .rev()
            .find(|item| item.id.as_ref().unwrap().version >= minimum)
            .and_then(|item| item.id.as_ref().map(PackageId::display));
    }
    let exact = reference.exact_version()?;
    options
        .iter()
        .find(|item| item.id.as_ref().unwrap().version == exact)
        .or_else(|| {
            options
                .iter()
                .rev()
                .find(|item| item.id.as_ref().unwrap().version > exact)
        })
        .and_then(|item| item.id.as_ref().map(PackageId::display))
}

fn missing_references(
    packages: &[VarPackage],
    index: &HashMap<String, Vec<&VarPackage>>,
) -> BTreeSet<String> {
    packages
        .iter()
        .filter(|item| item.valid)
        // VaM registers meta.json dependencies when it builds the package
        // library.  Content references are useful for repair, but omitting
        // declared dependencies made the old report disagree with VaM's own
        // missing-package log.
        .flat_map(|package| {
            package
                .content_refs
                .iter()
                .chain(package.declared_refs.keys())
        })
        .filter(|raw| !is_obvious_non_var_dependency(raw))
        .filter_map(|raw| parse_reference(raw))
        .filter(|reference| resolve_reference(reference, index).is_none())
        .map(|reference| reference.raw)
        .collect()
}

fn package_key(id: &PackageId) -> String {
    id.display().to_ascii_lowercase()
}

fn load_vam_log(path: &Path) -> Result<VamLogData> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("cannot read VaM log: {}", path.display()))?;
    let mut result = VamLogData::default();
    for line in text.lines() {
        if let Some(raw) = line
            .trim_start_matches(['!', '>', ' '])
            .strip_prefix("Clothing item ")
            .and_then(|value| value.strip_suffix(" is missing"))
            && resource_url_regex().is_match(raw)
        {
            result.missing_resource_urls.insert(raw.to_string());
        }
        if let Some(after_missing) = line
            .split_once("Missing addon package ")
            .map(|(_, value)| value)
            && let Some((dependency, owner_with_suffix)) = after_missing.split_once(" that package")
            && let Some(owner) = owner_with_suffix.strip_suffix(" depends on")
        {
            result
                .missing_by_owner
                .entry(owner.trim().to_ascii_lowercase())
                .or_default()
                .insert(dependency.trim().to_string());
        }
        if let Some(after) = line
            .split_once("Exception during process of meta.json from package ")
            .map(|(_, value)| value)
            && let Some((package, error)) = after.split_once(':')
        {
            let package = package.trim().to_ascii_lowercase();
            if error.contains("Size mismatch between central header") {
                result.header_mismatch_packages.insert(package.clone());
            }
            result.corrupt_packages.insert(package);
        }
    }
    Ok(result)
}

fn newest_var_write_time(root: &Path) -> Option<std::time::SystemTime> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("var"))
        })
        .filter_map(|entry| entry.metadata().ok()?.modified().ok())
        .max()
}

fn locate_vam_log(root: &Path, explicit: Option<&Path>) -> Result<(Option<PathBuf>, Vec<String>)> {
    if let Some(path) = explicit {
        if !path.is_file() {
            bail!("VaM log does not exist: {}", path.display());
        }
        return Ok((Some(path.to_path_buf()), Vec::new()));
    }
    let mut candidates = Vec::new();
    if let Some(vam_root) = root.parent() {
        candidates.push(vam_root.join("output_log.txt"));
        candidates.push(vam_root.join("logs").join("output_log.txt"));
    }
    if let Some(user_profile) = std::env::var_os("USERPROFILE") {
        candidates.push(
            PathBuf::from(user_profile)
                .join("AppData")
                .join("LocalLow")
                .join("MeshedVR")
                .join("VaM")
                .join("output_log.txt"),
        );
    }
    if let Some(path) = candidates.into_iter().find(|path| path.is_file()) {
        return Ok((Some(path), Vec::new()));
    }
    Ok((
        None,
        vec![
            "VaM output_log.txt was not found automatically. In VaM, use Package Manager > Rescan Packages, then rerun with --vam-log <path>."
                .to_string(),
        ],
    ))
}

fn vam_log_freshness(root: &Path, log_path: &Path) -> Result<Option<String>> {
    let log_time = fs::metadata(log_path)?.modified()?;
    let Some(newest_var) = newest_var_write_time(root) else {
        return Ok(None);
    };
    if log_time < newest_var {
        Ok(Some(
            "VaM log is older than at least one VAR in AddonPackages. Do not treat it as authoritative until VaM has been opened and Package Manager > Rescan Packages has completed."
                .to_string(),
        ))
    } else {
        Ok(None)
    }
}

fn package_dependencies(package: &VarPackage, vam_log: Option<&VamLogData>) -> BTreeSet<String> {
    let mut dependencies = package
        .content_refs
        .iter()
        .chain(package.declared_refs.keys())
        .filter(|dependency| !is_obvious_non_var_dependency(dependency))
        .cloned()
        .collect::<BTreeSet<_>>();
    if let (Some(id), Some(log)) = (&package.id, vam_log) {
        if let Some(logged) = log.missing_by_owner.get(&package_key(id)) {
            dependencies.extend(
                logged
                    .iter()
                    .filter(|dependency| !is_obvious_non_var_dependency(dependency))
                    .cloned(),
            );
        }
    }
    dependencies
}

fn resolve_reference_excluding<'a>(
    reference: &PackageRef,
    index: &'a HashMap<String, Vec<&'a VarPackage>>,
    excluded: &HashSet<String>,
    strict_exact: bool,
) -> Option<&'a VarPackage> {
    let options = index.get(&reference.family())?;
    let available = |item: &&VarPackage| {
        item.id
            .as_ref()
            .is_some_and(|id| !excluded.contains(&package_key(id)))
    };
    if reference.selector == "latest" {
        return options.iter().rev().find(|item| available(item)).copied();
    }
    if let Some(minimum) = reference
        .selector
        .strip_prefix("min")
        .and_then(|value| value.parse::<u32>().ok())
    {
        return options
            .iter()
            .rev()
            .find(|item| available(item) && item.id.as_ref().unwrap().version >= minimum)
            .copied();
    }
    let exact = reference.exact_version()?;
    options
        .iter()
        .find(|item| available(item) && item.id.as_ref().unwrap().version == exact)
        .copied()
        .or_else(|| {
            (!strict_exact).then(|| {
                options
                    .iter()
                    .rev()
                    .find(|item| available(item) && item.id.as_ref().unwrap().version > exact)
                    .copied()
            })?
        })
}

/// An exact non-plugin reference may move to a newer installed version.  A
/// `latest` reference normally needs no rewrite, except when VaM's case-
/// sensitive package lookup disagrees with the locally installed creator or
/// package casing (for example `HiJoker` vs `HiJoKer`).  In that narrow case
/// rewrite it to the concrete installed package ID that VaM can load.
fn is_safe_non_plugin_relink(reference: &PackageRef, resolved: &str) -> bool {
    if reference.exact_version().is_some() {
        return true;
    }
    if reference.selector != "latest" {
        return false;
    }
    let Some(installed) = parse_reference(resolved) else {
        return false;
    };
    (reference.creator != installed.creator || reference.package != installed.package)
        && reference.creator.eq_ignore_ascii_case(&installed.creator)
        && reference.package.eq_ignore_ascii_case(&installed.package)
}

fn report_dir(root: &Path, requested: Option<&Path>) -> PathBuf {
    requested
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.parent().unwrap_or(root).join("VaMVarReports"))
}

fn print_write_warning(backup: Option<&Path>) {
    eprintln!("CAUTION: This command can change files in AddonPackages.");
    if let Some(path) = backup {
        eprintln!("VaMender restore-point directory: {}", path.display());
    }
    eprintln!(
        "Keep a separate, tested full backup; do not rely on tool restore points as your only copy."
    );
    eprintln!(
        "VaMender is provided AS IS without warranty. You are responsible for data, licensing, and all results."
    );
}

fn write_reports(
    out: &Path,
    packages: &[VarPackage],
    index: &HashMap<String, Vec<&VarPackage>>,
    taken: &[String],
    extra_required: &[String],
) -> Result<()> {
    fs::create_dir_all(out)?;
    let missing = missing_references(packages, index);
    let invalid: Vec<_> = packages.iter().filter(|item| !item.valid).collect();
    let missing_members = missing_resource_members(packages, index);
    let mut required = vec![format!(
        "Library: {} VARs; {} valid providers; {} invalid archives.",
        packages.len(),
        index.values().map(Vec::len).sum::<usize>(),
        invalid.len()
    )];
    required.push(format!(
        "Missing dependencies after local version resolution: {}.",
        missing.len()
    ));
    required.push(format!(
        "Static package-member mismatches: {} reference(s). These are diagnostic candidates, not confirmed runtime failures; VaMender never deletes or invents internal content from this evidence.",
        missing_members.len()
    ));
    if !missing_members.is_empty() {
        required.push(
            "After VaM Package Manager > Rescan Packages, run `plan <AddonPackages>` so fresh VaM log warnings can identify the members that actually failed at runtime."
                .to_string(),
        );
    }
    if !invalid.is_empty() {
        required.push(
            "Restore a known-good copy of each invalid archive before relying on it in VaM."
                .to_string(),
        );
        for package in invalid.iter().take(20) {
            required.push(format!(
                "INVALID: {} :: {}",
                package.relative.display(),
                package.issues.join("; ")
            ));
        }
    }
    required.extend(extra_required.iter().cloned());
    if missing.is_empty() {
        required.push("No unresolved package IDs were found in scanned text content.".to_string());
    } else {
        required.push(
            "Unresolved IDs are listed one per line in missing_dependencies.txt. Acquire only the packages you are authorized to obtain, then use VaM Package Manager > Rescan Packages and run `plan <AddonPackages>`."
                .to_string(),
        );
    }
    let mut action_log = if taken.is_empty() {
        vec!["No live VAR files were changed.".to_string()]
    } else {
        taken.to_vec()
    };
    action_log.push(format!("Reports written: {}", out.display()));
    fs::write(out.join("actions_taken.txt"), action_log.join("\n") + "\n")?;
    fs::write(out.join("actions_required.txt"), required.join("\n") + "\n")?;
    fs::write(
        out.join("missing_dependencies.txt"),
        missing.into_iter().collect::<Vec<_>>().join("\n") + "\n",
    )?;
    Ok(())
}

fn sha256(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn backup_var(source: &Path, root: &Path, backup_root: &Path, operation: &str) -> Result<PathBuf> {
    let relative = source
        .strip_prefix(root)
        .context("backup source is outside AddonPackages")?;
    let digest = sha256(source)?;
    let destination = backup_root.join("files").join(format!(
        "{}--{}",
        &digest[..16],
        source.file_name().unwrap().to_string_lossy()
    ));
    fs::create_dir_all(destination.parent().unwrap())?;
    if !destination.exists() {
        fs::copy(source, &destination)?;
    }
    if sha256(&destination)? != digest {
        bail!(
            "backup checksum verification failed: {}",
            destination.display()
        );
    }
    let record = BackupRecord {
        operation: operation.to_string(),
        source: source.display().to_string(),
        relative_path: relative.to_string_lossy().replace('\\', "/"),
        backup: destination.display().to_string(),
        sha256: digest,
    };
    let mut manifest = OpenOptions::new()
        .create(true)
        .append(true)
        .open(backup_root.join("manifest.jsonl"))?;
    serde_json::to_writer(&mut manifest, &record)?;
    manifest.write_all(b"\n")?;
    Ok(destination)
}

fn atomic_rewrite(
    package: &VarPackage,
    meta: Option<&Value>,
    replacements: &HashMap<String, String>,
    root: &Path,
    backup: &Path,
    operation: &str,
) -> Result<()> {
    let _backup = backup_var(&package.path, root, backup, operation)?;
    let parent = package.path.parent().context("VAR has no parent folder")?;
    let temporary = NamedTempFile::new_in(parent)?;
    let temp_path = temporary.path().to_path_buf();
    let source = File::open(&package.path)?;
    let mut archive = match ZipArchive::new(source) {
        Ok(archive) => archive,
        Err(error) => {
            #[cfg(windows)]
            {
                rewrite_from_tar_compatibility_archive(package, meta, replacements, &temporary)
                    .with_context(|| format!("cannot rewrite compatibility archive: {error}"))?;
                verify_zip(&temp_path)?;
                return replace_live_var(&temp_path, &package.path);
            }
            #[cfg(not(windows))]
            return Err(error.into());
        }
    };
    {
        let output = temporary.reopen()?;
        let mut writer = ZipWriter::new(output);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        let mut wrote_meta = false;
        for index in 0..archive.len() {
            let mut member = archive.by_index(index)?;
            let name = member.name().to_string();
            if member.is_dir() {
                writer.add_directory(name, options)?;
                continue;
            }
            writer.start_file(name.clone(), options)?;
            if package.meta_name.as_deref() == Some(name.as_str()) && meta.is_some() {
                let output_meta = meta.unwrap();
                writer.write_all(serde_json::to_string_pretty(output_meta)?.as_bytes())?;
                writer.write_all(b"\n")?;
                wrote_meta = true;
            } else if is_text_member(&name) && !replacements.is_empty() {
                let mut raw = Vec::new();
                member.read_to_end(&mut raw)?;
                if let Ok(text) = String::from_utf8(raw.clone()) {
                    let edited = rewrite_reference_text(&text, replacements);
                    writer.write_all(edited.as_bytes())?;
                } else {
                    writer.write_all(&raw)?;
                }
            } else {
                io::copy(&mut member, &mut writer)?;
            }
        }
        if meta.is_some() && !wrote_meta {
            writer.start_file("meta.json", options)?;
            writer.write_all(serde_json::to_string_pretty(meta.unwrap())?.as_bytes())?;
            writer.write_all(b"\n")?;
        }
        writer.finish()?;
    }
    verify_zip(&temp_path)?;
    replace_live_var(&temp_path, &package.path)
}

fn replace_live_var(temp_path: &Path, destination: &Path) -> Result<()> {
    let displaced = destination.with_extension("var.vamender-displaced");
    fs::rename(destination, &displaced)?;
    if let Err(error) = fs::rename(temp_path, destination) {
        let _ = fs::rename(&displaced, destination);
        return Err(error.into());
    }
    fs::remove_file(displaced)?;
    Ok(())
}

#[cfg(windows)]
fn rewrite_from_tar_compatibility_archive(
    package: &VarPackage,
    meta: Option<&Value>,
    replacements: &HashMap<String, String>,
    temporary: &NamedTempFile,
) -> Result<()> {
    let listing = std::process::Command::new("tar.exe")
        .arg("-tf")
        .arg(&package.path)
        .output()
        .context("cannot enumerate compatibility archive with tar.exe")?;
    if !listing.status.success() {
        bail!("tar.exe could not enumerate this compatibility archive")
    }
    let listing =
        String::from_utf8(listing.stdout).context("tar.exe returned non-UTF-8 member names")?;
    let output = temporary.reopen()?;
    let mut writer = ZipWriter::new(output);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let mut wrote_meta = false;
    for raw_name in listing.lines() {
        let name = raw_name.trim().replace('\\', "/");
        if name.is_empty() || name.ends_with('/') {
            continue;
        }
        if name.starts_with('/') || name.split('/').any(|part| part == "..") {
            bail!("unsafe member path in compatibility archive: {name}");
        }
        writer.start_file(name.clone(), options)?;
        if package.meta_name.as_deref() == Some(name.as_str()) && meta.is_some() {
            let output_meta = meta.unwrap();
            writer.write_all(serde_json::to_string_pretty(output_meta)?.as_bytes())?;
            writer.write_all(b"\n")?;
            wrote_meta = true;
            continue;
        }
        let member = std::process::Command::new("tar.exe")
            .arg("-xOf")
            .arg(&package.path)
            .arg(&name)
            .output()
            .with_context(|| format!("cannot read {name} with tar.exe"))?;
        if !member.status.success() {
            bail!("tar.exe could not extract {name}");
        }
        if is_text_member(&name) && !replacements.is_empty() {
            if let Ok(text) = String::from_utf8(member.stdout.clone()) {
                writer.write_all(rewrite_reference_text(&text, replacements).as_bytes())?;
            } else {
                writer.write_all(&member.stdout)?;
            }
        } else {
            writer.write_all(&member.stdout)?;
        }
    }
    if meta.is_some() && !wrote_meta {
        writer.start_file("meta.json", options)?;
        writer.write_all(serde_json::to_string_pretty(meta.unwrap())?.as_bytes())?;
        writer.write_all(b"\n")?;
    }
    writer.finish()?;
    Ok(())
}

fn verify_zip(path: &Path) -> Result<()> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    for index in 0..archive.len() {
        let mut member = archive.by_index(index)?;
        io::copy(&mut member, &mut io::sink())?;
    }
    Ok(())
}

fn rewrite_reference_text(text: &str, replacements: &HashMap<String, String>) -> String {
    reference_regex()
        .replace_all(text, |captures: &regex::Captures| {
            let original = captures.name("id").unwrap().as_str();
            replacements
                .get(original)
                .cloned()
                .unwrap_or_else(|| original.to_string())
        })
        .into_owned()
}

fn resource_bytes_match(old: &VarPackage, newest: &VarPackage) -> Result<bool> {
    let old_file = File::open(&old.path)?;
    let new_file = File::open(&newest.path)?;
    let mut old_zip = ZipArchive::new(old_file)?;
    let mut new_zip = ZipArchive::new(new_file)?;
    for index in 0..old_zip.len() {
        let mut old_member = old_zip.by_index(index)?;
        if old_member.is_dir() {
            continue;
        }
        let name = old_member.name().replace('\\', "/");
        if !(name.starts_with("Custom/") || name.starts_with("Saves/")) {
            continue;
        }
        let mut new_member = match new_zip.by_name(&name) {
            Ok(member) => member,
            Err(_) => return Ok(false),
        };
        let mut old_hash = Sha256::new();
        let mut new_hash = Sha256::new();
        io::copy(&mut old_member, &mut HashWriter(&mut old_hash))?;
        io::copy(&mut new_member, &mut HashWriter(&mut new_hash))?;
        if old_hash.finalize() != new_hash.finalize() {
            return Ok(false);
        }
    }
    Ok(true)
}

struct HashWriter<'a>(&'a mut Sha256);
impl Write for HashWriter<'_> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.0.update(buffer);
        Ok(buffer.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn migrate_metadata(meta: &Value, replacements: &HashMap<String, String>) -> Result<Option<Value>> {
    let mut changed = meta.clone();
    let dependencies = match changed
        .get_mut("dependencies")
        .and_then(Value::as_object_mut)
    {
        Some(value) => value,
        None => return Ok(None),
    };
    let mut output = Map::new();
    let mut dirty = false;
    for (name, payload) in dependencies.iter() {
        let target = replacements.get(name).unwrap_or(name);
        if target != name {
            dirty = true;
        }
        if let Some(previous) = output.get(target) {
            if previous != payload && !is_empty_payload(previous) && !is_empty_payload(payload) {
                bail!("metadata would merge two non-empty dependency payloads into {target}");
            }
            if is_empty_payload(previous) && !is_empty_payload(payload) {
                output.insert(target.clone(), payload.clone());
            }
        } else {
            output.insert(target.clone(), payload.clone());
        }
    }
    if dirty {
        *dependencies = output;
        Ok(Some(changed))
    } else {
        Ok(None)
    }
}

/// Return the old exact IDs that cannot be migrated because a retained VAR's
/// metadata would merge two *non-empty* dependency payloads into one latest
/// ID.  Keeping those old versions is safer than choosing one payload or
/// abandoning every unrelated migration.
fn metadata_conflicting_replacement_keys(
    meta: &Value,
    replacements: &HashMap<String, String>,
) -> BTreeSet<String> {
    let Some(dependencies) = meta.get("dependencies").and_then(Value::as_object) else {
        return BTreeSet::new();
    };
    let mut seen: HashMap<String, (String, bool)> = HashMap::new();
    let mut blocked = BTreeSet::new();
    for (name, payload) in dependencies {
        let target = replacements.get(name).unwrap_or(name);
        let non_empty = !is_empty_payload(payload);
        if let Some((previous_name, previous_non_empty)) = seen.get(target) {
            if previous_name != name && *previous_non_empty && non_empty {
                if replacements.contains_key(previous_name) {
                    blocked.insert(previous_name.clone());
                }
                if replacements.contains_key(name) {
                    blocked.insert(name.clone());
                }
            }
        } else {
            seen.insert(target.clone(), (name.clone(), non_empty));
        }
    }
    blocked
}

fn filter_candidates_for_metadata_conflicts<'a>(
    packages: &'a [VarPackage],
    candidates: &mut Vec<(&'a VarPackage, &'a VarPackage)>,
    required: &mut Vec<String>,
) {
    // Removing a candidate can resolve a collision. Repeat until the remaining
    // replacement map is internally safe, and record every blocked old VAR in
    // the plan so apply and dry-run agree.
    loop {
        let replacements: HashMap<String, String> = candidates
            .iter()
            .map(|(old, newest)| {
                (
                    old.id.as_ref().unwrap().display(),
                    newest.id.as_ref().unwrap().display(),
                )
            })
            .collect();
        let archived: HashSet<PathBuf> =
            candidates.iter().map(|(old, _)| old.path.clone()).collect();
        let mut conflicts: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for source in packages
            .iter()
            .filter(|package| package.valid && !archived.contains(&package.path))
        {
            if let Some(meta) = &source.meta {
                let blocked = metadata_conflicting_replacement_keys(meta, &replacements);
                if !blocked.is_empty() {
                    conflicts.insert(source.relative.display().to_string(), blocked);
                }
            }
        }
        if conflicts.is_empty() {
            break;
        }
        let blocked: BTreeSet<String> = conflicts
            .values()
            .flat_map(|values| values.iter().cloned())
            .collect();
        let newest_by_old: HashMap<String, String> = candidates
            .iter()
            .map(|(old, newest)| {
                (
                    old.id.as_ref().unwrap().display(),
                    newest.id.as_ref().unwrap().display(),
                )
            })
            .collect();
        for (source, keys) in conflicts {
            for old_id in keys {
                if let Some(new_id) = newest_by_old.get(&old_id) {
                    required.push(format!(
                        "KEEP {old_id} -> {new_id} (non-empty metadata dependency payload conflict in {source})"
                    ));
                }
            }
        }
        let before = candidates.len();
        candidates.retain(|(old, _)| !blocked.contains(&old.id.as_ref().unwrap().display()));
        if candidates.len() == before {
            // This should be unreachable because `blocked` is assembled from
            // the current replacement keys. Do not spin if a malformed input
            // ever violates that assumption.
            break;
        }
    }
}

fn is_empty_payload(value: &Value) -> bool {
    value.is_null() || value.as_object().is_some_and(Map::is_empty)
}

fn version_label_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^(?:pre-)?v\d+(?:\.\d+){2,}$").unwrap())
}

fn is_obvious_non_var_dependency(value: &str) -> bool {
    let normalized = value.trim().to_ascii_lowercase();
    let Some(reference) = parse_reference(value) else {
        // VaM will try every meta.json dependency key as a package ID, but a
        // key outside the package grammar cannot ever be a satisfiable VAR.
        return true;
    };
    if version_label_regex().is_match(&normalized) {
        return true;
    }
    if normalized.contains("defaultmat_")
        || normalized.contains("_material.")
        || normalized.contains("_material_")
        || normalized.contains("entries.count")
        || normalized.contains("breathentries.length")
        || normalized.contains(".val-")
    {
        return true;
    }
    let udim_like = reference
        .exact_version()
        .is_some_and(|version| (1001..=1999).contains(&version));
    udim_like
        && (normalized.starts_with("base.spec.")
            || normalized.starts_with("base.gloss.")
            || [
                "_diffuse.",
                "_glossiness.",
                "_normal.",
                "_roughness.",
                "_opacity.",
                "_specular.",
                "_height.",
                "_basecolor.",
                "_sim.",
            ]
            .iter()
            .any(|suffix| normalized.contains(suffix)))
}

fn choose_license(
    package: &VarPackage,
    explicit: Option<&str>,
    non_interactive: bool,
    leave_all_unknown: &mut bool,
) -> Result<Option<String>> {
    if *leave_all_unknown {
        return Ok(None);
    }
    if let Some(license) = explicit {
        if license.eq_ignore_ascii_case("leave-all") {
            *leave_all_unknown = true;
            return Ok(None);
        }
        if LICENSES.contains(&license) {
            return Ok(Some(license.to_string()));
        }
        bail!("unsupported VaM license: {license}");
    }
    if non_interactive {
        return Ok(None);
    }
    println!(
        "\n{} needs a new meta.json, but its original license is unknown.",
        package.relative.display()
    );
    println!("  A) Leave this and all remaining unknown-license VARs unchanged");
    println!("  0) Leave unchanged (default; recommended when license is unknown)");
    for (index, license) in LICENSES.iter().enumerate() {
        println!(" {:>2}) {}", index + 1, license);
    }
    print!("Choose license or press Enter to leave unchanged: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice = input.trim();
    if choice.is_empty() || choice == "0" {
        return Ok(None);
    }
    if choice.eq_ignore_ascii_case("a") {
        *leave_all_unknown = true;
        return Ok(None);
    }
    let index: usize = choice.parse().context("enter a license number")?;
    Ok(LICENSES.get(index - 1).map(|value| value.to_string()))
}

fn repaired_metadata(
    package: &VarPackage,
    license: Option<String>,
    force_stale: bool,
) -> Result<Option<Value>> {
    let mut meta = match &package.meta {
        Some(value) => value.clone(),
        None => {
            let license = match license {
                Some(value) => value,
                None => return Ok(None),
            };
            let id = package
                .id
                .as_ref()
                .context("cannot rebuild meta.json for an invalid VAR file name")?;
            json!({"name": id.display(), "creatorName": id.creator, "packageName": id.display(), "licenseType": license, "description": "", "dependencies": {}})
        }
    };
    let object = meta
        .as_object_mut()
        .context("metadata root must be an object")?;
    let dependencies = object
        .entry("dependencies")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .context("metadata dependencies must be an object")?;
    let content: BTreeSet<_> = package
        .content_refs
        .iter()
        .filter_map(|raw| parse_reference(raw).map(|reference| reference.raw))
        .collect();
    for dependency in &content {
        dependencies
            .entry(dependency.clone())
            .or_insert_with(|| json!({}));
    }
    let stale: Vec<_> = dependencies
        .keys()
        .filter(|key| {
            let payload = &dependencies[*key];
            // Empty material/parameter labels are never loadable VARs and are
            // responsible for a large class of VaM's fake "Missing addon
            // package" messages. Remove those even if the same label appears
            // in a material JSON member. A non-plugin's empty metadata-only
            // package key is also removable: it carries no nested dependency
            // information and no explicit package URL/field in content uses
            // it. Plugins are always preserved as strict dependencies.
            (is_empty_payload(payload) && is_obvious_non_var_dependency(key))
                || (parse_reference(key).is_some()
                    && !content.contains(*key)
                    && is_empty_payload(payload)
                    && (!package.is_plugin || force_stale))
        })
        .cloned()
        .collect();
    for dependency in stale {
        dependencies.remove(&dependency);
    }
    Ok(Some(meta))
}

fn run_inspect(arguments: InspectArgs) -> Result<()> {
    println!(
        "Scanning {} in parallel{}...",
        arguments.root.display(),
        if arguments.deep {
            " with full CRC validation"
        } else {
            ""
        }
    );
    let packages = scan_library(&arguments.root, arguments.deep)?;
    let index = providers(&packages);
    let out = report_dir(&arguments.root, arguments.out.as_deref());
    write_reports(
        &out,
        &packages,
        &index,
        &[format!(
            "Inspection completed: {} VARs scanned.",
            packages.len()
        )],
        &[],
    )?;
    println!(
        "Complete: {}. Read actions_required.txt and missing_dependencies.txt.",
        out.display()
    );
    Ok(())
}

fn run_repair_with_mode(arguments: RepairArgs) -> Result<()> {
    println!(
        "Scanning {} in parallel (changed VARs are fully verified before replacement)...",
        arguments.root.display()
    );
    let mut packages = scan_library(&arguments.root, false)?;
    let out = report_dir(&arguments.root, arguments.out.as_deref());
    let mut required = Vec::new();
    let (vam_log_path, mut log_notes) = locate_vam_log(&arguments.root, None)?;
    let vam_log = match vam_log_path.as_deref() {
        Some(path) => match vam_log_freshness(&arguments.root, path)? {
            Some(note) => {
                log_notes.push(note);
                log_notes.push(
                    "VaM archive-repack candidates were skipped because the log is stale. Rescan packages in VaM, then rerun repair."
                        .to_string(),
                );
                None
            }
            None => {
                log_notes.push(format!("Using fresh VaM package log: {}.", path.display()));
                Some(load_vam_log(path)?)
            }
        },
        None => None,
    };
    required.append(&mut log_notes);

    let (filename_plans, mut filename_required) = plan_filename_repairs(&packages);
    let filename_plan_count = filename_plans.len();
    required.append(&mut filename_required);
    let mut filename_renamed = 0;
    let mut filename_duplicates = 0;
    let mut filename_failed = 0;
    let mut filename_actions = Vec::new();
    if arguments.apply && !filename_plans.is_empty() {
        let backup = arguments
            .backup
            .as_deref()
            .context("--backup is required with --apply")?;
        print_write_warning(Some(backup));
        fs::create_dir_all(backup)?;
        let filename_outcome = apply_filename_repairs(&filename_plans, &arguments.root, backup);
        filename_renamed = filename_outcome.renamed;
        filename_duplicates = filename_outcome.duplicates_archived;
        filename_failed = filename_outcome.failed;
        filename_actions = filename_outcome.actions;
        required.extend(filename_outcome.required);
        drop(filename_plans);
        packages = scan_library(&arguments.root, false)?;
    } else if !arguments.apply {
        filename_actions.extend(filename_plans.iter().map(|plan| {
            format!(
                "FILENAME PLAN: {} -> {}.var ({:?})",
                plan.package.relative.display(),
                plan.canonical_id.display(),
                plan.kind
            )
        }));
    }

    let index = providers(&packages);
    let mut plans = Vec::new();
    let mut leave_all_unknown = false;
    for package in packages.iter().filter(|item| item.valid) {
        let license = if package.meta.is_none() {
            choose_license(
                package,
                arguments.license.as_deref(),
                arguments.non_interactive,
                &mut leave_all_unknown,
            )?
        } else {
            None
        };
        let metadata = repaired_metadata(package, license, false)?
            .filter(|meta| package.meta.as_ref() != Some(meta));
        let repack_for_vam = package.id.as_ref().is_some_and(|id| {
            vam_log
                .as_ref()
                .is_some_and(|log| log.header_mismatch_packages.contains(&package_key(id)))
        });
        if package.meta.is_none() && metadata.is_none() {
            required.push(format!(
                "License decision required: {} has no usable meta.json.",
                package.relative.display()
            ));
        }
        if metadata.is_some() || repack_for_vam {
            plans.push(RepairWork {
                package,
                meta: metadata,
                repack_for_vam,
            });
        }
    }
    let metadata_count = plans.iter().filter(|work| work.meta.is_some()).count();
    let repack_count = plans.iter().filter(|work| work.repack_for_vam).count();
    let mut taken = vec![format!(
        "Repair plan: {filename_plan_count} filename repair(s); {metadata_count} metadata rewrite(s); {repack_count} VaM archive repackage(s); apply={}",
        arguments.apply
    )];
    taken.extend(filename_actions);
    if !arguments.apply {
        required.push(
            "Dry run only. Re-run with --apply --backup <folder> after reviewing this report."
                .to_string(),
        );
        write_reports(&out, &packages, &index, &taken, &required)?;
        println!(
            "Dry run complete: {filename_plan_count} filename repair(s), {metadata_count} metadata rewrite(s), {repack_count} archive repackage(s) planned. Reports: {}",
            out.display()
        );
        return Ok(());
    }
    let backup = arguments
        .backup
        .as_ref()
        .context("--backup is required with --apply")?;
    if filename_plan_count == 0 {
        print_write_warning(Some(backup));
    }
    fs::create_dir_all(backup)?;
    let mut applied = 0;
    let mut failed = 0;
    let mut repacked = 0;
    for work in plans {
        let operation = if work.repack_for_vam {
            "vam-header-repack"
        } else {
            "meta-repair"
        };
        match atomic_rewrite(
            work.package,
            work.meta.as_ref(),
            &HashMap::new(),
            &arguments.root,
            backup,
            operation,
        ) {
            Ok(()) => {
                applied += 1;
                if work.repack_for_vam {
                    repacked += 1;
                }
            }
            Err(error) => {
                failed += 1;
                required.push(format!(
                    "FAILED repair: {} :: {error}",
                    work.package.relative.display()
                ));
            }
        }
    }
    taken.push(format!(
        "Repair result: {filename_renamed} filename rename(s), {filename_duplicates} malformed duplicate(s) archived, {applied} archive rewrite(s) applied ({repacked} VaM archive repackage(s)); {} failed.",
        filename_failed + failed
    ));
    println!("Post-repair verification scan (fast parallel mode)...");
    let post_packages = scan_library(&arguments.root, false)?;
    let post_index = providers(&post_packages);
    write_reports(&out, &post_packages, &post_index, &taken, &required)?;
    println!(
        "Repair complete: {applied} applied, {failed} failed. Reports: {}",
        out.display()
    );
    Ok(())
}

fn run_repair(arguments: RepairArgs) -> Result<()> {
    run_repair_with_mode(arguments)
}

fn run_migrate_with_mode(arguments: MigrationArgs) -> Result<()> {
    println!(
        "Scanning {} in parallel (candidates are fully verified before archive)...",
        arguments.root.display()
    );
    let packages = scan_library(&arguments.root, false)?;
    let index = providers(&packages);
    let out = report_dir(&arguments.root, arguments.out.as_deref());
    let mut required = vec!["Version migration only proceeds when the newer VAR contains byte-identical Custom/Saves resources and neither package is a plugin (unless explicitly allowed).".to_string()];
    let mut taken = Vec::new();
    let mut candidates: Vec<(&VarPackage, &VarPackage)> = Vec::new();
    for versions in index.values() {
        if versions.len() < 2 {
            continue;
        }
        let newest = versions.last().unwrap();
        for old in &versions[..versions.len() - 1] {
            let same_resources = match resource_bytes_match(old, newest) {
                Ok(value) => value,
                Err(error) => {
                    required.push(format!(
                        "KEEP {} -> {} (archive/resource comparison failed: {error})",
                        old.id.as_ref().unwrap().display(),
                        newest.id.as_ref().unwrap().display()
                    ));
                    false
                }
            };
            if old.is_plugin || newest.is_plugin || !same_resources {
                required.push(format!(
                    "KEEP {} -> {} (plugin or resource compatibility not proven)",
                    old.id.as_ref().unwrap().display(),
                    newest.id.as_ref().unwrap().display()
                ));
            } else {
                candidates.push((*old, *newest));
            }
        }
    }
    filter_candidates_for_metadata_conflicts(&packages, &mut candidates, &mut required);
    if !arguments.apply {
        taken.push(format!(
            "Migration dry run: {} old-version candidate(s) passed the current safety gate.",
            candidates.len()
        ));
        required.push("Dry run only. Re-run with --apply --backup <folder> only after reviewing these candidates.".to_string());
        write_reports(&out, &packages, &index, &taken, &required)?;
        println!(
            "Migration plan complete: {} candidate(s). Reports: {}",
            candidates.len(),
            out.display()
        );
        return Ok(());
    }
    let backup = arguments
        .backup
        .as_ref()
        .context("--backup is required with --apply")?;
    print_write_warning(Some(backup));
    fs::create_dir_all(backup)?;
    let replacements: HashMap<String, String> = candidates
        .iter()
        .map(|(old, newest)| {
            (
                old.id.as_ref().unwrap().display(),
                newest.id.as_ref().unwrap().display(),
            )
        })
        .collect();
    let archived: HashSet<PathBuf> = candidates.iter().map(|(old, _)| old.path.clone()).collect();
    let mut source_updates: Vec<(&VarPackage, Option<Value>)> = Vec::new();
    for source in packages
        .iter()
        .filter(|package| package.valid && !archived.contains(&package.path))
    {
        let needs_content = source
            .content_refs
            .iter()
            .any(|reference| replacements.contains_key(reference));
        let metadata = match &source.meta {
            Some(meta) => migrate_metadata(meta, &replacements)?,
            None => None,
        };
        if needs_content || metadata.is_some() {
            source_updates.push((source, metadata));
        }
    }
    let mut rewritten = 0;
    let mut rewrite_failed = false;
    for (source, metadata) in &source_updates {
        if let Err(error) = atomic_rewrite(
            source,
            metadata.as_ref(),
            &replacements,
            &arguments.root,
            backup,
            "version-migration-reference-rewrite",
        ) {
            required.push(format!(
                "FAILED migration rewrite: {} :: {error}",
                source.relative.display()
            ));
            rewrite_failed = true;
            break;
        }
        rewritten += 1;
    }
    let post = scan_library(&arguments.root, false)?;
    let post_index = providers(&post);
    if rewrite_failed {
        required.push("No old VARs were archived because a reference rewrite failed. Restore from manifest.jsonl if you want to undo completed rewrites.".to_string());
        taken.push(format!(
            "Migration stopped after {rewritten} source VAR rewrite(s); zero old VARs archived."
        ));
        write_reports(&out, &post, &post_index, &taken, &required)?;
        bail!(
            "migration stopped before archive; see {}",
            out.join("actions_required.txt").display()
        );
    }
    let still_used = post
        .iter()
        .filter(|source| !archived.contains(&source.path))
        .any(|source| {
            source
                .content_refs
                .iter()
                .any(|reference| replacements.contains_key(reference))
                || source
                    .declared_refs
                    .keys()
                    .any(|reference| replacements.contains_key(reference))
        });
    if still_used {
        required.push("No old VARs were archived: the post-rewrite scan still found an exact old-version reference.".to_string());
        taken.push(format!(
            "Migration rewrote {rewritten} source VAR(s); zero old VARs archived."
        ));
        write_reports(&out, &post, &post_index, &taken, &required)?;
        bail!(
            "post-rewrite check blocked archiving; see {}",
            out.join("actions_required.txt").display()
        );
    }
    let mut archived_count = 0;
    for (old, _) in candidates {
        backup_var(&old.path, &arguments.root, backup, "old-version-archive")?;
        fs::remove_file(&old.path)?;
        archived_count += 1;
    }
    let final_packages = scan_library(&arguments.root, false)?;
    let final_index = providers(&final_packages);
    taken.push(format!("Migration result: {rewritten} source VAR(s) rewritten; {archived_count} old VAR(s) archived."));
    write_reports(&out, &final_packages, &final_index, &taken, &required)?;
    println!(
        "Migration complete: {rewritten} source VAR(s) rewritten; {archived_count} old VAR(s) archived. Reports: {}",
        out.display()
    );
    Ok(())
}

fn run_migrate(arguments: MigrationArgs) -> Result<()> {
    run_migrate_with_mode(arguments)
}

fn can_drop_empty_metadata_dependency(package: &VarPackage, dependency: &str) -> bool {
    !package.is_plugin
        && package
            .declared_refs
            .get(dependency)
            .is_some_and(is_empty_payload)
        && !package.content_refs.contains(dependency)
}

fn write_prune_plan_reports(
    out: &Path,
    actions: &[String],
    required: &[String],
    missing: &BTreeSet<String>,
) -> Result<()> {
    fs::create_dir_all(out)?;
    fs::write(out.join("actions_taken.txt"), actions.join("\n") + "\n")?;
    fs::write(out.join("actions_required.txt"), required.join("\n") + "\n")?;
    fs::write(
        out.join("missing_dependencies.txt"),
        missing.iter().cloned().collect::<Vec<_>>().join("\n") + "\n",
    )?;
    Ok(())
}

/// Apply the part of the VaM-aware plan that can be proven safe locally:
///
/// * non-plugin exact references can move only to a locally installed newer
///   version, with both metadata and textual resource URLs rewritten;
/// * empty non-plugin metadata-only keys are removed; and
/// * packages still in the dependency failure closure are backed up and
///   removed from AddonPackages.  They are deliberately archived rather than
///   deleted, so the manifest can restore them.
///
/// When a fresh VaM log is available, it is captured before the preceding
/// repair stage in `run_all`, so safe rewrites made in that stage do not make
/// its runtime evidence unusable.  Without one, the closure uses only direct
/// metadata and resource URLs found in the installed VARs; stale VaM-only
/// observations are never reused.
fn run_dependency_closure(
    root: &Path,
    out: &Path,
    backup: &Path,
    vam_log: Option<&VamLogData>,
) -> Result<()> {
    let packages = scan_library(root, false)?;
    let index = providers(&packages);
    let mut required = vec![if vam_log.is_some() {
        "Dependency closure used VaM's fresh rescan log. Script packages remain exact-version only; non-plugin links may use a newer local version."
            .to_string()
    } else {
        "VaM log was stale or unavailable. Dependency closure used only direct local metadata and resource URLs; stale VaM-only observations were ignored."
            .to_string()
    }];
    let mut taken = Vec::new();

    let relink_plan = build_dependency_relink_plan(&packages, &index, vam_log);
    let relink_result = apply_dependency_relink_plan(&packages, &relink_plan, root, backup);
    required.extend(relink_result.required);
    taken.extend(relink_result.taken);
    let relinked = relink_result.relinked;
    let metadata_cleaned = relink_result.metadata_cleaned;
    if relink_result.failed {
        let current = scan_library(root, false)?;
        let current_index = providers(&current);
        required.push(
            "No dependency-closure VARs were archived because one or more safe relinks did not complete. Restore from manifest.jsonl if you want to undo completed relinks."
                .to_string(),
        );
        write_reports(out, &current, &current_index, &taken, &required)?;
        bail!(
            "dependency relink stopped before quarantine; see {}",
            out.join("actions_required.txt").display()
        );
    }

    let packages = scan_library(root, false)?;
    let index = providers(&packages);
    let (reasons, triggering_missing) = dependency_failure_closure(&packages, &index, vam_log);

    let candidates: Vec<_> = packages
        .iter()
        .enumerate()
        .filter(|(position, _)| !reasons[*position].is_empty())
        .collect();
    let candidate_count = candidates.len();
    for (position, package) in &candidates {
        if let Err(error) = backup_var(&package.path, root, backup, "dependency-closure-quarantine")
        {
            required.push(format!(
                "FAILED QUARANTINE BACKUP {} :: {error}",
                package.relative.display()
            ));
            let current = scan_library(root, false)?;
            let current_index = providers(&current);
            write_reports(out, &current, &current_index, &taken, &required)?;
            bail!("dependency closure stopped before removing a VAR");
        }
        if let Err(error) = fs::remove_file(&package.path) {
            required.push(format!(
                "FAILED QUARANTINE REMOVE {} :: {error}",
                package.relative.display()
            ));
            let current = scan_library(root, false)?;
            let current_index = providers(&current);
            write_reports(out, &current, &current_index, &taken, &required)?;
            bail!("dependency closure stopped after creating a restore point");
        }
        taken.push(format!(
            "QUARANTINED: {} :: {}",
            package.relative.display(),
            reasons[*position]
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join("; ")
        ));
    }
    taken.insert(
        0,
        format!(
            "Dependency closure result: {relinked} non-plugin VAR(s) relinked; {metadata_cleaned} metadata-only dependency cleanup(s); {candidate_count} VAR(s) archived after checksum-verified backup."
        ),
    );
    if !triggering_missing.is_empty() {
        required.push(format!(
            "{} missing package ID(s) triggered the archived dependency closure. They no longer have a retained installed dependent.",
            triggering_missing.len()
        ));
    }
    let final_packages = scan_library(root, false)?;
    let final_index = providers(&final_packages);
    write_reports(out, &final_packages, &final_index, &taken, &required)?;
    println!(
        "Dependency closure complete: {relinked} relinked; {candidate_count} archived. Reports: {}",
        out.display()
    );
    Ok(())
}

fn run_plan(arguments: OptimizeArgs) -> Result<()> {
    println!(
        "Building a read-only VaM-authoritative cleanup plan. No VAR will be changed, moved, or deleted."
    );
    println!(
        "Scanning {} for dependency and VaM-log evidence...",
        arguments.root.display()
    );
    let packages = scan_library(&arguments.root, false)?;
    let index = providers(&packages);
    let missing_members = missing_resource_members(&packages, &index);
    let (filename_plans, filename_plan_required) = plan_filename_repairs(&packages);
    let repairable_filenames: HashSet<PathBuf> = filename_plans
        .iter()
        .map(|plan| plan.package.path.clone())
        .collect();
    let (vam_log_path, mut log_notes) =
        locate_vam_log(&arguments.root, arguments.vam_log.as_deref())?;
    let vam_log = match vam_log_path.as_deref() {
        Some(path) => match vam_log_freshness(&arguments.root, path)? {
            Some(note) => {
                log_notes.push(note);
                log_notes.push(format!(
                    "Ignoring stale VaM log at {}. Run VaM's Package Manager > Rescan Packages after the newest VAR changes, then rerun plan.",
                    path.display()
                ));
                None
            }
            None => {
                log_notes.push(format!("Using fresh VaM package log: {}.", path.display()));
                Some(load_vam_log(path)?)
            }
        },
        None => None,
    };
    let mut reasons = vec![BTreeSet::<String>::new(); packages.len()];
    let mut malformed_names = Vec::new();
    let mut log_corrupt_not_found = Vec::new();
    let mut header_repack = BTreeSet::new();
    let ids_in_library: HashSet<String> = packages
        .iter()
        .filter_map(|package| package.id.as_ref().map(package_key))
        .collect();

    for (position, package) in packages.iter().enumerate() {
        if !package.valid {
            reasons[position].insert(format!("invalid archive: {}", package.issues.join("; ")));
        }
        match &package.id {
            Some(id) => {
                if vam_log
                    .as_ref()
                    .is_some_and(|log| log.header_mismatch_packages.contains(&package_key(id)))
                {
                    header_repack.insert(package.relative.display().to_string());
                } else if vam_log
                    .as_ref()
                    .is_some_and(|log| log.corrupt_packages.contains(&package_key(id)))
                {
                    reasons[position]
                        .insert("VaM rejected this archive while loading meta.json".to_string());
                }
            }
            None if !repairable_filenames.contains(&package.path) => {
                malformed_names.push(package.relative.display().to_string())
            }
            None => {}
        }
    }
    if let Some(log) = &vam_log {
        log_corrupt_not_found = log
            .corrupt_packages
            .iter()
            .filter(|id| !ids_in_library.contains(*id))
            .cloned()
            .collect();
    }

    let mut relinks = BTreeSet::new();
    let mut metadata_drops = BTreeSet::new();
    let mut triggering_missing = BTreeSet::new();
    loop {
        let removed_ids: HashSet<String> = packages
            .iter()
            .enumerate()
            .filter(|(position, package)| !reasons[*position].is_empty() && package.id.is_some())
            .map(|(_, package)| package_key(package.id.as_ref().unwrap()))
            .collect();
        let mut changed = false;
        for (position, package) in packages.iter().enumerate() {
            if !package.valid || package.id.is_none() || !reasons[position].is_empty() {
                continue;
            }
            for raw in package_dependencies(package, vam_log.as_ref()) {
                let Some(reference) = parse_reference(&raw) else {
                    if vam_log.as_ref().is_some_and(|log| {
                        log.missing_by_owner
                            .get(&package_key(package.id.as_ref().unwrap()))
                            .is_some_and(|values| values.contains(&raw))
                    }) {
                        triggering_missing.insert(raw.clone());
                        reasons[position]
                            .insert(format!("VaM reports unparseable missing dependency {raw}"));
                        changed = true;
                    }
                    continue;
                };
                let provider = resolve_reference_excluding(
                    &reference,
                    &index,
                    &removed_ids,
                    package.is_plugin,
                );
                let Some(provider) = provider else {
                    if can_drop_empty_metadata_dependency(package, &raw) {
                        metadata_drops.insert(format!(
                            "DROP METADATA: {} :: {}",
                            package.relative.display(),
                            raw
                        ));
                    } else {
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
                    continue;
                };
                let resolved = provider.id.as_ref().unwrap().display();
                if !package.is_plugin
                    && is_safe_non_plugin_relink(&reference, &resolved)
                    && resolved != reference.raw
                {
                    relinks.insert(format!(
                        "RELINK NON-PLUGIN: {} :: {} -> {}",
                        package.relative.display(),
                        reference.raw,
                        resolved
                    ));
                }
            }
        }
        if !changed {
            break;
        }
    }

    let removed_count = reasons.iter().filter(|value| !value.is_empty()).count();
    let retained = packages.len() - removed_count;
    let invalid_count = packages.iter().filter(|package| !package.valid).count();
    let mut actions = vec![
        "PLAN ONLY: no VAR was changed, moved, quarantined, or deleted.".to_string(),
        format!(
            "Dependency closure: {} VARs scanned; {} would remain; {} would be quarantined.",
            packages.len(),
            retained,
            removed_count
        ),
        format!(
            "Potential safe actions before quarantine: {} filename repair(s); {} VaM archive repackage(s); {} non-plugin version relink(s); {} empty metadata dependency removal(s).",
            filename_plans.len(),
            header_repack.len(),
            relinks.len(),
            metadata_drops.len()
        ),
    ];
    for plan in &filename_plans {
        actions.push(format!(
            "FILENAME REPAIR: {} -> {}.var ({:?})",
            plan.package.relative.display(),
            plan.canonical_id.display(),
            plan.kind
        ));
    }
    for package in &header_repack {
        actions.push(format!(
            "REPACKAGE: {package} :: VaM reported a ZIP central/local-header mismatch; repair can rebuild it without changing its content."
        ));
    }
    actions.extend(relinks);
    actions.extend(metadata_drops);
    for (position, package) in packages.iter().enumerate() {
        if !reasons[position].is_empty() {
            actions.push(format!(
                "QUARANTINE: {} :: {}",
                package.relative.display(),
                reasons[position]
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("; ")
            ));
        }
    }

    let mut required = vec![
        "This is a read-only cleanup plan. It intentionally proposes quarantine, not deletion, and does not perform any relink or metadata rewrite."
            .to_string(),
        "Script packages are strict: a numeric dependency must exist at that exact version. Only non-plugin packages are eligible for a newer-local-version relink proposal."
            .to_string(),
        format!(
            "Invalid archives detected by the scanner: {invalid_count}. VaM header-mismatch archives queued for safe repack: {}. Other VaM-rejected archives: {}.",
            header_repack.len(),
            vam_log.as_ref().map_or(0, |log| log.corrupt_packages.len().saturating_sub(log.header_mismatch_packages.len()))
        ),
    ];
    if !malformed_names.is_empty() {
        required.push(format!(
            "{} VAR filename(s) do not follow VaM's integer version convention and cannot provide dependencies: {}",
            malformed_names.len(),
            malformed_names.iter().take(20).cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    required.extend(filename_plan_required);
    required.push(format!(
        "Static package-member mismatches: {} reference(s). They are not automatically rewritten because unused content can contain dormant references.",
        missing_members.len()
    ));
    if let Some(log) = &vam_log {
        let confirmed_members = logged_missing_resource_members(log, &index);
        required.push(format!(
            "VaM-confirmed missing internal members in the fresh log: {}.",
            confirmed_members.len()
        ));
        required.extend(confirmed_members);
    }
    if !log_corrupt_not_found.is_empty() {
        required.push(format!(
            "{} corrupt package ID(s) in the VaM log were not found by the current library scan (possibly removed or nonstandard filename).",
            log_corrupt_not_found.len()
        ));
    }
    required.append(&mut log_notes);
    required.push(format!(
        "{} unresolved dependency ID(s) triggered this closure; they are listed one per line in missing_dependencies.txt.",
        triggering_missing.len()
    ));
    let out = report_dir(&arguments.root, arguments.out.as_deref());
    write_prune_plan_reports(&out, &actions, &required, &triggering_missing)?;
    println!(
        "Plan complete: {retained} retained, {removed_count} quarantine candidates. Reports: {}",
        out.display()
    );
    Ok(())
}

fn stage_summary(stage: &Path, preferred_prefix: &str) -> String {
    fs::read_to_string(stage.join("actions_taken.txt"))
        .ok()
        .and_then(|text| {
            text.lines()
                .find(|line| line.starts_with(preferred_prefix))
                .or_else(|| text.lines().next())
                .map(str::to_string)
        })
        .unwrap_or_else(|| format!("{preferred_prefix} report was not produced."))
}

fn stage_has_required_prefix(stage: &Path, prefix: &str) -> bool {
    fs::read_to_string(stage.join("actions_required.txt"))
        .map(|text| text.lines().any(|line| line.starts_with(prefix)))
        .unwrap_or(false)
}

fn run_all(arguments: RunArgs) -> Result<()> {
    print_write_warning(Some(&arguments.backup));
    fs::create_dir_all(&arguments.backup)?;
    let out = report_dir(&arguments.root, arguments.out.as_deref());
    let details = out.join("_details");
    fs::create_dir_all(&details)?;
    let repair_out = details.join("repair");
    let closure_out = details.join("dependency-closure");
    let migrate_out = details.join("migrate");
    let final_out = details.join("final-check");
    println!(
        "Running automatic safe cleanup: metadata repair/repack, safe relinks, dependency closure, conservative migration, then deep verification."
    );

    let mut taken = vec![
        "Automatic run: no review gates were used; every changed or archived VAR was backed up first."
            .to_string(),
    ];
    let mut required = Vec::new();
    // Capture VaM's runtime evidence before the repair stage changes any
    // archive timestamp.  It stays valid for the following safe relinks and
    // dependency closure because those changes can only remove problems VaM
    // had just reported, never invent new runtime references.
    let (located_log, _) = locate_vam_log(&arguments.root, None)?;
    let (closure_log, closure_note) = match located_log {
        Some(path) => match vam_log_freshness(&arguments.root, &path)? {
            Some(note) => (None, Some(note)),
            None => (Some(load_vam_log(&path)?), None),
        },
        None => (None, Some("No VaM output_log.txt was found; dependency closure was skipped. Rescan packages in VaM, then run again.".to_string())),
    };
    if let Some(log) = &closure_log {
        let packages = scan_library(&arguments.root, false)?;
        let index = providers(&packages);
        let confirmed_members = logged_missing_resource_members(log, &index);
        if !confirmed_members.is_empty() {
            required.push(format!(
                "Fresh VaM log confirmed {} missing internal package member(s). VaMender left their source references unchanged because fabricating or deleting content is unsafe.",
                confirmed_members.len()
            ));
            required.extend(confirmed_members);
        }
    }
    let mut repair_failed = false;
    let repair = run_repair_with_mode(RepairArgs {
        root: arguments.root.clone(),
        out: Some(repair_out.clone()),
        apply: true,
        backup: Some(arguments.backup.clone()),
        license: arguments.license.clone(),
        non_interactive: true,
    });
    match repair {
        Ok(()) => {
            taken.push(format!(
                "Repair: {}",
                stage_summary(&repair_out, "Repair result:")
            ));
            if stage_has_required_prefix(&repair_out, "FAILED repair:") {
                repair_failed = true;
                required.push(
                    "Repair stage reported one or more failed VAR rewrites; migration was skipped."
                        .to_string(),
                );
            }
        }
        Err(error) => {
            repair_failed = true;
            required.push(format!("Repair stage failed: {error}"));
        }
    }

    if !repair_failed {
        match run_dependency_closure(
            &arguments.root,
            &closure_out,
            &arguments.backup,
            closure_log.as_ref(),
        ) {
            Ok(()) => taken.push(format!(
                "Dependency closure: {}",
                stage_summary(&closure_out, "Dependency closure result:")
            )),
            Err(error) => required.push(format!("Dependency closure stopped safely: {error}")),
        }
        if let Some(note) = closure_note {
            required.push(format!(
                "{note} The dependency closure therefore used only locally provable references."
            ));
        }
    } else {
        required
            .push("Dependency closure was skipped because the repair stage failed.".to_string());
    }

    if !repair_failed {
        match run_migrate_with_mode(MigrationArgs {
            root: arguments.root.clone(),
            out: Some(migrate_out.clone()),
            apply: true,
            backup: Some(arguments.backup.clone()),
        }) {
            Ok(()) => taken.push(format!(
                "Migration: {}",
                stage_summary(&migrate_out, "Migration result:")
            )),
            Err(error) => required.push(format!("Migration stage stopped safely: {error}")),
        }
    } else {
        required.push("Migration was skipped because the repair stage failed.".to_string());
    }

    match run_inspect(InspectArgs {
        root: arguments.root.clone(),
        out: Some(final_out.clone()),
        deep: true,
    }) {
        Ok(()) => taken.push(format!(
            "Final verification: {}",
            stage_summary(&final_out, "Inspection completed:")
        )),
        Err(error) => required.push(format!("Final verification failed: {error}")),
    }
    let final_missing = final_out.join("missing_dependencies.txt");
    if final_missing.is_file() {
        fs::copy(&final_missing, out.join("missing_dependencies.txt"))?;
    }
    required.push(
        "Use VaM Package Manager > Rescan Packages, then run `plan <AddonPackages>` to evaluate VaM's fresh runtime evidence after this automatic run."
            .to_string(),
    );
    required.push(format!(
        "Detailed stage reports are in {}. The three top-level reports are the normal handoff.",
        details.display()
    ));
    fs::write(out.join("actions_taken.txt"), taken.join("\n") + "\n")?;
    fs::write(out.join("actions_required.txt"), required.join("\n") + "\n")?;
    println!(
        "Automatic run complete. Read {} and then rescan packages in VaM.",
        out.join("actions_required.txt").display()
    );
    if required.iter().any(|line| {
        line.starts_with("Repair stage failed")
            || line.starts_with("Repair stage reported")
            || line.starts_with("Dependency closure stopped")
            || line.starts_with("Migration stage stopped")
            || line.starts_with("Final verification failed")
    }) {
        bail!(
            "automatic run completed final verification with a blocked stage; see {}",
            out.display()
        );
    }
    Ok(())
}

fn safe_restore_target(root: &Path, relative: &str) -> Option<PathBuf> {
    let relative = Path::new(relative);
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::Prefix(_)
                    | std::path::Component::RootDir
            )
        })
    {
        return None;
    }
    Some(root.join(relative))
}

fn run_restore(arguments: RestoreArgs) -> Result<()> {
    print_write_warning(arguments.manifest.parent());
    if !arguments.root.is_dir() {
        bail!(
            "AddonPackages folder does not exist: {}",
            arguments.root.display()
        );
    }
    let manifest = fs::read_to_string(&arguments.manifest).with_context(|| {
        format!(
            "cannot read restore manifest {}",
            arguments.manifest.display()
        )
    })?;
    let mut records = Vec::new();
    for (line_number, line) in manifest.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        records.push(
            serde_json::from_str::<BackupRecord>(line)
                .with_context(|| format!("invalid manifest record at line {}", line_number + 1))?,
        );
    }
    if arguments.last == Some(0) {
        bail!("--last must be at least 1");
    }
    let conflict_root = arguments
        .manifest
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("restore-conflicts")
        .join(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string(),
        );
    let mut restored = 0usize;
    let mut skipped = 0usize;
    let mut seen = HashSet::new();
    // A cumulative manifest may contain multiple restore points for one VAR.
    // The most recent record is the only sensible default when undoing the
    // latest cleanup.
    let records: Vec<_> = if let Some(last) = arguments.last {
        records.into_iter().rev().take(last).collect()
    } else {
        records.into_iter().rev().collect()
    };
    for record in records {
        if !seen.insert(record.relative_path.clone()) {
            continue;
        }
        let Some(target) = safe_restore_target(&arguments.root, &record.relative_path) else {
            eprintln!("SKIP unsafe restore target: {}", record.relative_path);
            skipped += 1;
            continue;
        };
        let source = PathBuf::from(&record.backup);
        if !source.is_file() || sha256(&source)? != record.sha256 {
            eprintln!("SKIP invalid restore source: {}", source.display());
            skipped += 1;
            continue;
        }
        if target.exists() {
            if !arguments.overwrite {
                eprintln!("SKIP existing VAR (use --overwrite): {}", target.display());
                skipped += 1;
                continue;
            }
            let conflict = conflict_root.join(&record.relative_path);
            if let Some(parent) = conflict.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&target, &conflict).with_context(|| {
                format!("cannot preserve restore conflict {}", target.display())
            })?;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&source, &target)
            .with_context(|| format!("cannot restore {}", target.display()))?;
        if sha256(&target)? != record.sha256 {
            bail!("restore checksum verification failed: {}", target.display());
        }
        restored += 1;
    }
    println!(
        "Restore complete: {restored} VAR(s) restored; {skipped} skipped.{}",
        if arguments.overwrite && conflict_root.exists() {
            format!(
                " Existing files were preserved in {}",
                conflict_root.display()
            )
        } else {
            String::new()
        }
    );
    Ok(())
}

fn dispatch(command: Command) -> Result<()> {
    match command {
        Command::Check(arguments) => run_inspect(arguments),
        Command::Plan(arguments) => run_plan(arguments),
        Command::Repair(arguments) => run_repair(arguments),
        Command::Migrate(arguments) => run_migrate(arguments),
        Command::Run(arguments) => run_all(arguments),
        Command::Restore(arguments) => run_restore(arguments),
        Command::SupportReport(arguments) => run_support_report(arguments),
        Command::InstallHost(arguments) => install_host(arguments),
        Command::UninstallHost(arguments) => uninstall_host(arguments),
        Command::StopHost => stop_installed_host(),
        Command::Host(_) => bail!("tray host must run on the process main thread"),
        Command::Bridge(arguments) => run_bridge(arguments),
    }
}

pub(crate) fn run() -> Result<()> {
    // A no-argument launch of an installed copy is a one-click tray restart.
    // Portable copies without host.json retain clap's normal help/error path.
    if std::env::args_os().len() == 1 && restart_installed_host()? {
        return Ok(());
    }
    // Windows notification-area objects require their message pump on the
    // process main thread. All archive commands still run on an explicitly
    // large worker stack to protect very large VaM libraries.
    let command = Cli::parse().command;
    if let Command::Host(arguments) = command {
        return run_tray_host(arguments);
    }
    std::thread::Builder::new()
        .name("vam-var-command".to_string())
        .stack_size(COMMAND_STACK_BYTES)
        .spawn(move || dispatch(command))
        .context("cannot start large-stack command worker")?
        .join()
        .map_err(|_| anyhow::anyhow!("VAR command worker panicked"))?
}

#[allow(dead_code)]
pub(crate) fn run_installed_tray_host() -> Result<()> {
    run_tray_host(installed_host_arguments()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_var(path: &Path, members: &[(&str, &str)]) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        for (name, text) in members {
            writer.start_file(*name, options)?;
            writer.write_all(text.as_bytes())?;
        }
        writer.finish()?;
        Ok(())
    }

    #[test]
    fn explicit_license_rebuilds_missing_metadata_with_whole_var_backup() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let root = temporary.path().join("AddonPackages");
        fs::create_dir(&root)?;
        let var = root.join("Author.Scene.1.var");
        write_var(
            &var,
            &[("Saves/scene/test.json", r#"{"dependency":"Other.Asset.2"}"#)],
        )?;
        let package = scan_package(&var, &root, false);
        assert!(package.valid);
        assert!(package.meta.is_none());
        let updated = repaired_metadata(&package, Some("CC BY".to_string()), false)?.unwrap();
        let backup = temporary.path().join("backup");
        atomic_rewrite(
            &package,
            Some(&updated),
            &HashMap::new(),
            &root,
            &backup,
            "test",
        )?;
        let mut archive = ZipArchive::new(File::open(&var)?)?;
        let mut metadata = String::new();
        archive
            .by_name("meta.json")?
            .read_to_string(&mut metadata)?;
        assert_eq!(
            serde_json::from_str::<Value>(&metadata)?["licenseType"],
            "CC BY"
        );
        assert!(backup.join("manifest.jsonl").is_file());
        Ok(())
    }

    #[test]
    fn exact_reference_falls_back_to_newer_local_var() {
        let reference = parse_reference("Author.Asset.1").unwrap();
        assert_eq!(reference.exact_version(), Some(1));
        assert!(is_safe_non_plugin_relink(&reference, "Author.Asset.2"));
        let case_mismatch = parse_reference("HiJoker.PleatedSkirt.latest").unwrap();
        assert!(is_safe_non_plugin_relink(
            &case_mismatch,
            "HiJoKer.PleatedSkirt.6"
        ));
        let normal_latest = parse_reference("Author.Asset.latest").unwrap();
        assert!(!is_safe_non_plugin_relink(&normal_latest, "Author.Asset.2"));
        assert!(parse_reference("Nokisaki.时崎狂三.latest").is_some());
        assert!(
            references_in_text(
                r#"{"url":"Nokisaki.时崎狂三.latest:/Custom/Hair/Female/test.vam"}"#
            )
            .contains("Nokisaki.时崎狂三.latest")
        );
    }

    #[test]
    fn metadata_repair_keeps_explicit_var_urls_but_removes_material_labels() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let root = temporary.path().join("AddonPackages");
        fs::create_dir(&root)?;
        let var = root.join("Author.Scene.1.var");
        write_var(
            &var,
            &[
                (
                    "meta.json",
                    r#"{"name":"Author.Scene.1","dependencies":{"Real.Asset.1":{},"base.spec.1001":{},"ears_0.12_low_defaultMat_Normal.1001":{}}}"#,
                ),
                (
                    "Saves/scene/test.json",
                    r#"{"url":"Real.Asset.1:/Custom/Assets/real.asset","material":"base.spec.1001","name":"ears_0.12_low_defaultMat_Normal.1001"}"#,
                ),
            ],
        )?;
        let package = scan_package(&var, &root, false);
        let repaired = repaired_metadata(&package, None, false)?.unwrap();
        let dependencies = repaired["dependencies"].as_object().unwrap();
        assert!(dependencies.contains_key("Real.Asset.1"));
        assert!(!dependencies.contains_key("base.spec.1001"));
        assert!(!dependencies.contains_key("ears_0.12_low_defaultMat_Normal.1001"));
        Ok(())
    }

    #[test]
    fn metadata_payload_conflict_blocks_only_the_colliding_old_versions() {
        let meta: Value = serde_json::from_str(
            r#"{"dependencies":{"JohnSaken.Freckles.1":{"preset":"a"},"JohnSaken.Freckles.2":{"preset":"b"},"Other.Asset.1":{}}}"#,
        )
        .unwrap();
        let replacements = HashMap::from([
            (
                "JohnSaken.Freckles.1".to_string(),
                "JohnSaken.Freckles.3".to_string(),
            ),
            (
                "JohnSaken.Freckles.2".to_string(),
                "JohnSaken.Freckles.3".to_string(),
            ),
        ]);
        assert_eq!(
            metadata_conflicting_replacement_keys(&meta, &replacements),
            BTreeSet::from([
                "JohnSaken.Freckles.1".to_string(),
                "JohnSaken.Freckles.2".to_string(),
            ])
        );
    }

    #[test]
    fn migration_rewrites_exact_references_then_archives_old_version() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let root = temporary.path().join("AddonPackages");
        fs::create_dir(&root)?;
        let metadata_one = r#"{"name":"Author.Asset.1","dependencies":{}}"#;
        let metadata_two = r#"{"name":"Author.Asset.2","dependencies":{}}"#;
        write_var(
            &root.join("Author.Asset.1.var"),
            &[
                ("meta.json", metadata_one),
                ("Custom/Assets/Author/item.txt", "same"),
            ],
        )?;
        write_var(
            &root.join("Author.Asset.2.var"),
            &[
                ("meta.json", metadata_two),
                ("Custom/Assets/Author/item.txt", "same"),
            ],
        )?;
        let scene_meta = r#"{"name":"Scene.Test.1","dependencies":{"Author.Asset.1":{}}}"#;
        let scene_json = r#"{"asset":"Author.Asset.1"}"#;
        write_var(
            &root.join("Scene.Test.1.var"),
            &[
                ("meta.json", scene_meta),
                ("Saves/scene/test.json", scene_json),
            ],
        )?;
        let output = temporary.path().join("reports");
        let backup = temporary.path().join("backup");
        run_migrate(MigrationArgs {
            root: root.clone(),
            out: Some(output),
            apply: true,
            backup: Some(backup),
        })?;
        assert!(!root.join("Author.Asset.1.var").exists());
        let mut archive = ZipArchive::new(File::open(root.join("Scene.Test.1.var"))?)?;
        let mut scene = String::new();
        archive
            .by_name("Saves/scene/test.json")?
            .read_to_string(&mut scene)?;
        assert!(scene.contains("Author.Asset.2"));
        Ok(())
    }

    #[test]
    fn dependency_closure_relinks_non_plugin_then_archives_and_restores_broken_var() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let root = temporary.path().join("AddonPackages");
        fs::create_dir(&root)?;
        write_var(
            &root.join("Author.Asset.2.var"),
            &[(
                "meta.json",
                r#"{"name":"Author.Asset.2","dependencies":{}}"#,
            )],
        )?;
        write_var(
            &root.join("Scene.Relink.1.var"),
            &[
                (
                    "meta.json",
                    r#"{"name":"Scene.Relink.1","dependencies":{"Author.Asset.1":{}}}"#,
                ),
                (
                    "Saves/scene/test.json",
                    r#"{"url":"Author.Asset.1:/Custom/item"}"#,
                ),
            ],
        )?;
        write_var(
            &root.join("Scene.Broken.1.var"),
            &[
                (
                    "meta.json",
                    r#"{"name":"Scene.Broken.1","dependencies":{}}"#,
                ),
                (
                    "Saves/scene/broken.json",
                    r#"{"url":"Missing.Asset.1:/Custom/item"}"#,
                ),
            ],
        )?;
        let output = temporary.path().join("reports");
        let backup = temporary.path().join("backup");
        run_dependency_closure(&root, &output, &backup, None)?;
        assert!(!root.join("Scene.Broken.1.var").exists());
        let mut archive = ZipArchive::new(File::open(root.join("Scene.Relink.1.var"))?)?;
        let mut scene = String::new();
        archive
            .by_name("Saves/scene/test.json")?
            .read_to_string(&mut scene)?;
        assert!(scene.contains("Author.Asset.2"));
        run_restore(RestoreArgs {
            root: root.clone(),
            manifest: backup.join("manifest.jsonl"),
            overwrite: false,
            last: None,
        })?;
        assert!(root.join("Scene.Broken.1.var").exists());
        Ok(())
    }

    #[test]
    fn filename_repair_archives_identical_malformed_duplicate() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let root = temporary.path().join("AddonPackages");
        fs::create_dir(&root)?;
        let canonical = root.join("Author.Asset.3.var");
        let malformed = root.join("Author.Asset.3_3.var");
        write_var(
            &canonical,
            &[
                (
                    "meta.json",
                    r#"{"creatorName":"Author","packageName":"Asset","licenseType":"CC BY","dependencies":{}}"#,
                ),
                ("Custom/Assets/Author/item.txt", "same"),
            ],
        )?;
        fs::copy(&canonical, &malformed)?;

        let packages = scan_library(&root, false)?;
        let (plans, required) = plan_filename_repairs(&packages);
        assert!(required.is_empty());
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].kind, FilenameRepairKind::ArchiveDuplicate);

        let backup = temporary.path().join("backup");
        let outcome = apply_filename_repairs(&plans, &root, &backup);
        assert_eq!(outcome.duplicates_archived, 1);
        assert_eq!(outcome.failed, 0);
        assert!(canonical.exists());
        assert!(!malformed.exists());
        assert!(backup.join("manifest.jsonl").exists());
        Ok(())
    }

    #[test]
    fn filename_repair_renames_unambiguous_download_suffix() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let root = temporary.path().join("AddonPackages");
        fs::create_dir(&root)?;
        let malformed = root.join("Author.Asset.7 (1).var");
        write_var(
            &malformed,
            &[(
                "meta.json",
                r#"{"creatorName":"Author","packageName":"Asset","licenseType":"CC BY","dependencies":{}}"#,
            )],
        )?;

        let packages = scan_library(&root, false)?;
        let (plans, required) = plan_filename_repairs(&packages);
        assert!(required.is_empty());
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].kind, FilenameRepairKind::Rename);

        let backup = temporary.path().join("backup");
        let outcome = apply_filename_repairs(&plans, &root, &backup);
        assert_eq!(outcome.renamed, 1);
        assert_eq!(outcome.failed, 0);
        assert!(root.join("Author.Asset.7.var").exists());
        assert!(!malformed.exists());
        Ok(())
    }

    #[test]
    fn reports_installed_provider_with_missing_referenced_member() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let root = temporary.path().join("AddonPackages");
        fs::create_dir(&root)?;
        write_var(
            &root.join("Provider.Clothes.3.var"),
            &[
                (
                    "meta.json",
                    r#"{"creatorName":"Provider","packageName":"Clothes","licenseType":"CC BY","dependencies":{}}"#,
                ),
                ("Custom/Clothing/Male/Provider/Present/Present.vam", "{}"),
            ],
        )?;
        write_var(
            &root.join("Scene.Owner.1.var"),
            &[
                (
                    "meta.json",
                    r#"{"creatorName":"Scene","packageName":"Owner","licenseType":"CC BY","dependencies":{"Provider.Clothes.latest":{}}}"#,
                ),
                (
                    "Saves/scene/test.json",
                    r#"{"clothing":[{"id":"Provider.Clothes.latest:/Custom/Clothing/Male/Provider/Missing/Missing.vam"}]}"#,
                ),
            ],
        )?;

        let packages = scan_library(&root, false)?;
        let index = providers(&packages);
        let missing = missing_resource_members(&packages, &index);
        assert_eq!(missing.len(), 1);
        assert!(missing[0].contains("Scene.Owner.1.var"));
        assert!(missing[0].contains("Provider.Clothes.3.var"));
        assert!(missing[0].contains("Missing.vam"));
        Ok(())
    }

    #[test]
    fn parses_and_diagnoses_vam_missing_clothing_warning() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let root = temporary.path().join("AddonPackages");
        fs::create_dir(&root)?;
        write_var(
            &root.join("VamEssentials.Super_Wet_Pack.3.var"),
            &[
                (
                    "meta.json",
                    r#"{"creatorName":"VamEssentials","packageName":"Super_Wet_Pack","licenseType":"CC BY","dependencies":{}}"#,
                ),
                ("Custom/Clothing/Male/VamEssentials/Present.vam", "{}"),
            ],
        )?;
        let log_path = temporary.path().join("output_log.txt");
        fs::write(
            &log_path,
            "!> Clothing item VamEssentials.Super_Wet_Pack.3:/Custom/Clothing/Male/VamEssentials/Super Wet Pack/Super Wet Penis 2/Super Wet Penis 2.vam is missing\n",
        )?;

        let packages = scan_library(&root, false)?;
        let index = providers(&packages);
        let log = load_vam_log(&log_path)?;
        let diagnosed = logged_missing_resource_members(&log, &index);
        assert_eq!(diagnosed.len(), 1);
        assert!(diagnosed[0].contains("VAM-CONFIRMED MISSING MEMBER"));
        assert!(diagnosed[0].contains("VamEssentials.Super_Wet_Pack.3.var"));
        assert!(diagnosed[0].contains("reacquire a known-good provider"));
        Ok(())
    }
}

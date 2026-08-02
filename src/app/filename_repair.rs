// SPDX-License-Identifier: MIT

use super::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum FilenameRepairKind {
    Rename,
    ArchiveDuplicate,
}

pub(super) struct FilenameRepairPlan<'a> {
    pub(super) package: &'a VarPackage,
    pub(super) canonical_id: PackageId,
    pub(super) destination: PathBuf,
    pub(super) kind: FilenameRepairKind,
}

#[derive(Default)]
pub(super) struct FilenameRepairOutcome {
    pub(super) renamed: usize,
    pub(super) duplicates_archived: usize,
    pub(super) failed: usize,
    pub(super) actions: Vec<String>,
    pub(super) required: Vec<String>,
}

fn malformed_version_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"^(?P<version>[0-9]+)(?:_[0-9]+|\.[0-9]+|\s*\([0-9]+\))$")
            .expect("valid malformed-version regex")
    })
}

fn valid_identity_component(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_alphanumeric() || matches!(character, '_' | '-'))
}

fn canonical_id_from_metadata(package: &VarPackage) -> Option<PackageId> {
    if package.id.is_some() || !package.valid {
        return None;
    }
    let meta = package.meta.as_ref()?;
    let creator = meta.get("creatorName")?.as_str()?.trim();
    let package_name = meta.get("packageName")?.as_str()?.trim();
    if !valid_identity_component(creator) || !valid_identity_component(package_name) {
        return None;
    }
    let file_name = package.path.file_name()?.to_str()?;
    let stem = file_name
        .strip_suffix(".var")
        .or_else(|| file_name.strip_suffix(".VAR"))?;
    let mut parts = stem.splitn(3, '.');
    let file_creator = parts.next()?;
    let file_package = parts.next()?;
    let malformed_version = parts.next()?;
    if !file_creator.eq_ignore_ascii_case(creator)
        || !file_package.eq_ignore_ascii_case(package_name)
    {
        return None;
    }
    let captures = malformed_version_regex().captures(malformed_version)?;
    let version = captures.name("version")?.as_str().parse().ok()?;
    Some(PackageId {
        creator: creator.to_string(),
        package: package_name.to_string(),
        version,
    })
}

pub(super) fn plan_filename_repairs<'a>(
    packages: &'a [VarPackage],
) -> (Vec<FilenameRepairPlan<'a>>, Vec<String>) {
    let mut providers_by_id: HashMap<String, Vec<&VarPackage>> = HashMap::new();
    for package in packages.iter().filter(|package| package.valid) {
        if let Some(id) = &package.id {
            providers_by_id
                .entry(package_key(id))
                .or_default()
                .push(package);
        }
    }

    let mut plans = Vec::new();
    let mut required = Vec::new();
    for package in packages {
        let Some(canonical_id) = canonical_id_from_metadata(package) else {
            continue;
        };
        let existing = providers_by_id
            .get(&package_key(&canonical_id))
            .cloned()
            .unwrap_or_default();
        if existing.len() > 1 {
            required.push(format!(
                "SKIP FILENAME: {} -> {}.var :: multiple canonical providers already exist",
                package.relative.display(),
                canonical_id.display()
            ));
            continue;
        }
        if let Some(canonical) = existing.first() {
            match (sha256(&package.path), sha256(&canonical.path)) {
                (Ok(source_hash), Ok(canonical_hash)) if source_hash == canonical_hash => {
                    plans.push(FilenameRepairPlan {
                        package,
                        canonical_id,
                        destination: canonical.path.clone(),
                        kind: FilenameRepairKind::ArchiveDuplicate,
                    });
                }
                (Ok(_), Ok(_)) => required.push(format!(
                    "SKIP FILENAME COLLISION: {} -> {} :: canonical file differs; neither file will be overwritten",
                    package.relative.display(),
                    canonical.relative.display()
                )),
                (Err(error), _) | (_, Err(error)) => required.push(format!(
                    "SKIP FILENAME: {} :: cannot verify canonical collision: {error}",
                    package.relative.display()
                )),
            }
            continue;
        }

        let destination = package
            .path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(format!("{}.var", canonical_id.display()));
        if destination.exists() {
            required.push(format!(
                "SKIP FILENAME COLLISION: {} -> {} :: destination exists but is not a registered canonical provider",
                package.relative.display(),
                destination.display()
            ));
            continue;
        }
        plans.push(FilenameRepairPlan {
            package,
            canonical_id,
            destination,
            kind: FilenameRepairKind::Rename,
        });
    }
    (plans, required)
}

pub(super) fn apply_filename_repairs(
    plans: &[FilenameRepairPlan<'_>],
    root: &Path,
    backup: &Path,
) -> FilenameRepairOutcome {
    let mut outcome = FilenameRepairOutcome::default();
    for plan in plans {
        let result = apply_filename_repair(plan, root, backup);
        match result {
            Ok(action) => {
                match plan.kind {
                    FilenameRepairKind::Rename => outcome.renamed += 1,
                    FilenameRepairKind::ArchiveDuplicate => outcome.duplicates_archived += 1,
                }
                outcome.actions.push(action);
            }
            Err(error) => {
                outcome.failed += 1;
                outcome.required.push(format!(
                    "FAILED FILENAME REPAIR: {} :: {error}",
                    plan.package.relative.display()
                ));
            }
        }
    }
    outcome
}

fn apply_filename_repair(
    plan: &FilenameRepairPlan<'_>,
    root: &Path,
    backup: &Path,
) -> Result<String> {
    let source_hash = sha256(&plan.package.path)?;
    backup_var(&plan.package.path, root, backup, "var-filename-repair")?;
    match plan.kind {
        FilenameRepairKind::ArchiveDuplicate => {
            if !plan.destination.is_file() || sha256(&plan.destination)? != source_hash {
                bail!("canonical duplicate changed after planning; source was left untouched");
            }
            fs::remove_file(&plan.package.path)?;
            Ok(format!(
                "FILENAME DUPLICATE ARCHIVED: {} -> {}",
                plan.package.relative.display(),
                plan.canonical_id.display()
            ))
        }
        FilenameRepairKind::Rename => {
            if plan.destination.exists() {
                bail!("canonical destination appeared after planning; source was left untouched");
            }
            fs::rename(&plan.package.path, &plan.destination)?;
            if sha256(&plan.destination)? != source_hash {
                let _ = fs::rename(&plan.destination, &plan.package.path);
                bail!("renamed VAR failed checksum verification and was moved back");
            }
            let destination_relative = plan
                .destination
                .strip_prefix(root)
                .unwrap_or(&plan.destination);
            Ok(format!(
                "FILENAME REPAIRED: {} -> {}",
                plan.package.relative.display(),
                destination_relative.display()
            ))
        }
    }
}

// SPDX-License-Identifier: MIT

use super::*;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(super) struct PackageId {
    pub(super) creator: String,
    pub(super) package: String,
    pub(super) version: u32,
}

impl PackageId {
    pub(super) fn display(&self) -> String {
        format!("{}.{}.{}", self.creator, self.package, self.version)
    }

    pub(super) fn family(&self) -> String {
        format!(
            "{}.{}",
            self.creator.to_lowercase(),
            self.package.to_lowercase()
        )
    }
}

#[derive(Debug, Clone)]
pub(super) struct PackageRef {
    pub(super) raw: String,
    pub(super) creator: String,
    pub(super) package: String,
    pub(super) selector: String,
}

impl PackageRef {
    pub(super) fn family(&self) -> String {
        format!(
            "{}.{}",
            self.creator.to_lowercase(),
            self.package.to_lowercase()
        )
    }

    pub(super) fn exact_version(&self) -> Option<u32> {
        self.selector.parse().ok()
    }
}

#[derive(Debug, Clone)]
pub(super) struct VarPackage {
    pub(super) path: PathBuf,
    pub(super) relative: PathBuf,
    pub(super) id: Option<PackageId>,
    pub(super) valid: bool,
    pub(super) issues: Vec<String>,
    pub(super) entries: BTreeSet<String>,
    pub(super) is_plugin: bool,
    pub(super) meta_name: Option<String>,
    pub(super) meta: Option<Value>,
    pub(super) content_refs: BTreeSet<String>,
    pub(super) resource_urls: BTreeSet<String>,
    pub(super) declared_refs: BTreeMap<String, Value>,
}

pub(super) struct RepairWork<'a> {
    pub(super) package: &'a VarPackage,
    pub(super) meta: Option<Value>,
    pub(super) repack_for_vam: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) struct BackupRecord {
    pub(super) operation: String,
    pub(super) source: String,
    pub(super) relative_path: String,
    pub(super) backup: String,
    pub(super) sha256: String,
}

#[derive(Default)]
pub(super) struct VamLogData {
    pub(super) missing_by_owner: BTreeMap<String, BTreeSet<String>>,
    pub(super) missing_resource_urls: BTreeSet<String>,
    pub(super) corrupt_packages: BTreeSet<String>,
    pub(super) header_mismatch_packages: BTreeSet<String>,
}

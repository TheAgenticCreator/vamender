// SPDX-License-Identifier: MIT

use super::*;
#[cfg(windows)]
use std::process::Command as ProcessCommand;

const GITHUB_SUPPORT_URL: &str =
    "https://github.com/TheAgenticCreator/vamender/issues/new?template=support_report.yml";
const SUPPORT_FILES: &[&str] = &[
    "README_FIRST.txt",
    "support_report.md",
    "package_issues.txt",
    "vam_package_issues.txt",
    "installed_packages.txt",
];

fn package_label(package: &VarPackage) -> String {
    package
        .id
        .as_ref()
        .map(PackageId::display)
        .or_else(|| {
            package
                .relative
                .file_name()
                .and_then(|value| value.to_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "unidentified VAR".to_string())
}

fn package_issue_lines(
    packages: &[VarPackage],
    index: &HashMap<String, Vec<&VarPackage>>,
    vam_log: Option<&VamLogData>,
) -> Vec<String> {
    let mut issues = BTreeSet::new();
    for package in packages {
        let owner = package_label(package);
        for issue in &package.issues {
            issues.insert(format!("INVALID OR DAMAGED: {owner} :: {issue}"));
        }
        if package.valid {
            for raw in package_dependencies(package, vam_log) {
                let Some(reference) = parse_reference(&raw) else {
                    continue;
                };
                if resolve_reference(&reference, index).is_none() {
                    issues.insert(format!("MISSING DEPENDENCY: {owner} -> {}", reference.raw));
                }
            }
        }
    }
    for issue in missing_resource_members(packages, index) {
        issues.insert(format!("STATIC MEMBER CHECK: {issue}"));
    }
    if let Some(log) = vam_log {
        for issue in logged_missing_resource_members(log, index) {
            issues.insert(issue);
        }
    }
    issues.into_iter().collect()
}

fn vam_issue_lines(log: Option<&VamLogData>, notices: &[String]) -> Vec<String> {
    let mut lines = vec![
        "VaMender extracts package-related identifiers only; the complete VaM log is never copied into this report."
            .to_string(),
    ];
    lines.extend(notices.iter().map(|notice| format!("NOTICE: {notice}")));
    let Some(log) = log else {
        lines.push("No VaM package log was available.".to_string());
        return lines;
    };
    for (owner, dependencies) in &log.missing_by_owner {
        for dependency in dependencies {
            lines.push(format!("MISSING PACKAGE: {owner} -> {dependency}"));
        }
    }
    for resource in &log.missing_resource_urls {
        lines.push(format!("MISSING INTERNAL RESOURCE: {resource}"));
    }
    for package in &log.header_mismatch_packages {
        lines.push(format!("ZIP HEADER MISMATCH: {package}"));
    }
    for package in &log.corrupt_packages {
        lines.push(format!("CORRUPT OR UNREADABLE PACKAGE: {package}"));
    }
    if lines.len() == 1 + notices.len() {
        lines.push("No recognized package-related VaM errors were extracted.".to_string());
    }
    lines
}

fn write_support_zip(out: &Path) -> Result<PathBuf> {
    let path = out.join("vamender-support-bundle.zip");
    let file = File::create(&path)
        .with_context(|| format!("cannot create support bundle {}", path.display()))?;
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for name in SUPPORT_FILES {
        let source = out.join(name);
        if !source.is_file() {
            continue;
        }
        writer.start_file(*name, options)?;
        writer.write_all(&fs::read(source)?)?;
    }
    writer.finish()?;
    Ok(path)
}

#[cfg(windows)]
fn open_github_support() -> Result<()> {
    ProcessCommand::new("rundll32.exe")
        .arg("url.dll,FileProtocolHandler")
        .arg(GITHUB_SUPPORT_URL)
        .spawn()
        .context("cannot open the GitHub support form")?;
    Ok(())
}

#[cfg(not(windows))]
fn open_github_support() -> Result<()> {
    bail!("--open-github is supported only on Windows; open {GITHUB_SUPPORT_URL} manually")
}

pub(super) fn run_support_report(arguments: SupportReportArgs) -> Result<()> {
    if arguments.open_github && !arguments.confirm_reviewed {
        bail!(
            "--open-github requires --confirm-reviewed after you inspect the generated files; VaMender never uploads diagnostics automatically"
        );
    }
    println!(
        "Creating a read-only VaMender support report from {}{}...",
        arguments.root.display(),
        if arguments.deep {
            " with full CRC validation"
        } else {
            ""
        }
    );
    let packages = scan_library(&arguments.root, arguments.deep)?;
    let index = providers(&packages);
    let out = arguments
        .out
        .clone()
        .unwrap_or_else(|| report_dir(&arguments.root, None).join("support"));
    fs::create_dir_all(&out)?;

    let (log_path, mut notices) = locate_vam_log(&arguments.root, arguments.vam_log.as_deref())?;
    if let Some(path) = &log_path
        && let Some(stale) = vam_log_freshness(&arguments.root, path)?
    {
        notices.push(stale);
    }
    let vam_log = log_path.as_deref().map(load_vam_log).transpose()?;
    let package_issues = package_issue_lines(&packages, &index, vam_log.as_ref());
    let vam_issues = vam_issue_lines(vam_log.as_ref(), &notices);
    let invalid_count = packages.iter().filter(|package| !package.valid).count();
    let missing_count = missing_references(&packages, &index).len();

    let readme = format!(
        "VAMENDER SUPPORT BUNDLE — REVIEW BEFORE SHARING\n\n\
This bundle was created locally. VaMender did not upload or transmit it.\n\
It contains package IDs and filenames that may reveal installed, paid, private,\n\
or otherwise sensitive content. Review every file before attaching the ZIP.\n\n\
Never attach VAR payloads, full VaM logs, backup manifests, absolute paths,\n\
credentials, tokens, private URLs, or content you do not have permission to share.\n\n\
Modified third-party VARs are for your local use only. Do not upload or\n\
redistribute them; local changes also alter Hub-recognized hashes. Preserve your\n\
independent backup and original packages.\n\n\
Report at: {GITHUB_SUPPORT_URL}\n\
Attach vamender-support-bundle.zip only after reviewing it.\n"
    );
    fs::write(out.join("README_FIRST.txt"), readme)?;

    let installed_note = if arguments.include_var_list {
        "Included by explicit --include-var-list request."
    } else {
        "Not included. Rerun with --include-var-list only if maintainers request it and you consent to disclosing package names."
    };
    let summary = format!(
        "# VaMender support report\n\n\
- VaMender version: {}\n\
- Release channel: beta\n\
- Platform family: {}\n\
- VARs scanned: {}\n\
- Valid providers: {}\n\
- Invalid or damaged VARs: {}\n\
- Unresolved package IDs: {}\n\
- Package issue lines: {}\n\
- VaM package log available: {}\n\
- Full installed VAR list: {}\n\n\
## Privacy boundary\n\n\
No VAR payload, absolute AddonPackages path, complete VaM log, backup manifest,\n\
credential, token, or private URL is intentionally included. Package IDs and\n\
filenames can still be sensitive; review all files before sharing.\n\n\
## Reporting\n\n\
Use the VaMender GitHub support form and attach the generated ZIP only after\n\
review. VaMender never submits or uploads diagnostics automatically.\n",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        packages.len(),
        index.values().map(Vec::len).sum::<usize>(),
        invalid_count,
        missing_count,
        package_issues.len(),
        if vam_log.is_some() { "yes" } else { "no" },
        installed_note,
    );
    fs::write(out.join("support_report.md"), summary)?;
    fs::write(
        out.join("package_issues.txt"),
        if package_issues.is_empty() {
            "No package issues were detected.\n".to_string()
        } else {
            package_issues.join("\n") + "\n"
        },
    )?;
    fs::write(
        out.join("vam_package_issues.txt"),
        vam_issues.join("\n") + "\n",
    )?;
    let installed_path = out.join("installed_packages.txt");
    if arguments.include_var_list {
        let mut installed = packages.iter().map(package_label).collect::<Vec<_>>();
        installed.sort();
        installed.dedup();
        fs::write(&installed_path, installed.join("\n") + "\n")?;
    } else {
        fs::write(
            &installed_path,
            "Installed package inventory not included. Rerun with --include-var-list only after reviewing the disclosure risk.\n",
        )?;
    }
    let bundle = write_support_zip(&out)?;

    println!("Support report written: {}", out.display());
    println!(
        "Review before sharing: {}",
        out.join("README_FIRST.txt").display()
    );
    println!("Bundle: {}", bundle.display());
    println!("No files were uploaded or transmitted.");
    if arguments.open_github {
        eprintln!(
            "Opening GitHub after explicit confirmation. VaMender will not attach or transmit the bundle."
        );
        open_github_support()?;
    } else {
        println!("GitHub support: {GITHUB_SUPPORT_URL}");
    }
    Ok(())
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
    fn support_report_extracts_package_issues_without_absolute_paths() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let root = temporary.path().join("AddonPackages");
        fs::create_dir(&root)?;
        write_var(
            &root.join("Scene.Owner.1.var"),
            &[
                (
                    "meta.json",
                    r#"{"name":"Scene.Owner.1","dependencies":{"Missing.Asset.2":{}}}"#,
                ),
                (
                    "Saves/scene/test.json",
                    r#"{"url":"Missing.Asset.2:/Custom/item"}"#,
                ),
            ],
        )?;
        let log = temporary.path().join("output_log.txt");
        fs::write(
            &log,
            "Missing addon package Missing.Asset.2 that package Scene.Owner.1 depends on\n",
        )?;
        let out = temporary.path().join("support");
        run_support_report(SupportReportArgs {
            root: root.clone(),
            out: Some(out.clone()),
            vam_log: Some(log),
            deep: false,
            include_var_list: true,
            open_github: false,
            confirm_reviewed: false,
        })?;

        let issues = fs::read_to_string(out.join("package_issues.txt"))?;
        let summary = fs::read_to_string(out.join("support_report.md"))?;
        assert!(issues.contains("Missing.Asset.2"));
        assert!(!issues.contains(&temporary.path().display().to_string()));
        assert!(!summary.contains(&temporary.path().display().to_string()));
        assert!(out.join("installed_packages.txt").is_file());
        assert!(out.join("vamender-support-bundle.zip").is_file());

        let bundle = File::open(out.join("vamender-support-bundle.zip"))?;
        let mut archive = zip::ZipArchive::new(bundle)?;
        assert_eq!(archive.len(), SUPPORT_FILES.len());
        assert!(archive.by_name("README_FIRST.txt").is_ok());
        assert!(archive.by_name("support_report.md").is_ok());
        assert!(archive.by_name("package_issues.txt").is_ok());
        assert!(archive.by_name("vam_package_issues.txt").is_ok());
        assert!(archive.by_name("installed_packages.txt").is_ok());
        Ok(())
    }

    #[test]
    fn support_report_opt_out_replaces_a_stale_inventory_with_a_marker() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let root = temporary.path().join("AddonPackages");
        let out = temporary.path().join("support");
        fs::create_dir(&root)?;
        fs::create_dir(&out)?;
        fs::write(out.join("installed_packages.txt"), "Private.Creator.1\n")?;

        run_support_report(SupportReportArgs {
            root,
            out: Some(out.clone()),
            vam_log: None,
            deep: false,
            include_var_list: false,
            open_github: false,
            confirm_reviewed: false,
        })?;

        let inventory = fs::read_to_string(out.join("installed_packages.txt"))?;
        assert!(inventory.contains("not included"));
        assert!(!inventory.contains("Private.Creator.1"));
        Ok(())
    }

    #[test]
    fn github_handoff_requires_review_confirmation() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let error = run_support_report(SupportReportArgs {
            root: temporary.path().to_path_buf(),
            out: None,
            vam_log: None,
            deep: false,
            include_var_list: false,
            open_github: true,
            confirm_reviewed: false,
        })
        .expect_err("GitHub handoff must require explicit review confirmation");
        assert!(error.to_string().contains("--confirm-reviewed"));
        Ok(())
    }
}

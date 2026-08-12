// SPDX-License-Identifier: MIT

use super::*;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use std::process::{Command as ProcessCommand, Stdio};

const TASK_NAME: &str = "VaMender Engine Host";
const HOST_EXECUTABLE_NAME: &str = "vamender-host.exe";

fn local_app_data() -> Result<PathBuf> {
    let value = std::env::var_os("LOCALAPPDATA")
        .context("LOCALAPPDATA is unavailable; VaMender requires a Windows user profile")?;
    Ok(PathBuf::from(value).join("VaMender"))
}

fn validate_install_paths(vam_root: &Path, backup: &Path) -> Result<(PathBuf, PathBuf, PathBuf)> {
    let root = fs::canonicalize(vam_root)
        .with_context(|| format!("cannot resolve VaM root {}", vam_root.display()))?;
    if !root.join("VaM.exe").is_file() {
        bail!("{} does not contain VaM.exe", root.display());
    }
    let packages = root.join("AddonPackages");
    if !packages.is_dir() {
        bail!("{} does not contain AddonPackages", root.display());
    }
    fs::create_dir_all(backup)
        .with_context(|| format!("cannot create backup directory {}", backup.display()))?;
    let backup = fs::canonicalize(backup)
        .with_context(|| format!("cannot resolve backup directory {}", backup.display()))?;
    if backup == packages || backup.starts_with(&packages) {
        bail!("backup directory must be outside {}", packages.display());
    }
    Ok((root, packages, backup))
}

#[cfg(windows)]
fn run_schtasks(arguments: &[&str]) -> Result<()> {
    let output = ProcessCommand::new("schtasks.exe")
        .args(arguments)
        .output()
        .context("cannot start Windows Task Scheduler command")?;
    if output.status.success() {
        return Ok(());
    }
    let error = String::from_utf8_lossy(&output.stderr);
    let standard = String::from_utf8_lossy(&output.stdout);
    bail!(
        "Windows Task Scheduler failed ({}): {}{}",
        output.status,
        standard.trim(),
        error.trim()
    );
}

#[cfg(not(windows))]
fn run_schtasks(_arguments: &[&str]) -> Result<()> {
    bail!("automatic VaM integration is supported only on Windows")
}

#[cfg(windows)]
fn run_reg(arguments: &[&str]) -> Result<()> {
    let output = ProcessCommand::new("reg.exe")
        .args(arguments)
        .output()
        .context("cannot start Windows Registry command")?;
    if output.status.success() {
        return Ok(());
    }
    let error = String::from_utf8_lossy(&output.stderr);
    let standard = String::from_utf8_lossy(&output.stdout);
    bail!(
        "Windows Registry command failed ({}): {}{}",
        output.status,
        standard.trim(),
        error.trim()
    );
}

#[cfg(not(windows))]
fn run_reg(_arguments: &[&str]) -> Result<()> {
    bail!("automatic VaM integration is supported only on Windows")
}

#[cfg(windows)]
fn start_host(executable: &Path) -> Result<()> {
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    ProcessCommand::new(executable)
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("cannot start installed engine {}", executable.display()))?;
    Ok(())
}

#[cfg(not(windows))]
fn start_host(_executable: &Path) -> Result<()> {
    bail!("automatic VaM integration is supported only on Windows")
}

#[cfg(windows)]
fn start_legacy_host(executable: &Path, packages: &Path, backup: &Path) -> Result<()> {
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    ProcessCommand::new(executable)
        .arg("host")
        .arg(packages)
        .arg("--backup")
        .arg(backup)
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("cannot start installed engine {}", executable.display()))?;
    Ok(())
}

#[cfg(not(windows))]
fn start_legacy_host(_executable: &Path, _packages: &Path, _backup: &Path) -> Result<()> {
    bail!("automatic VaM integration is supported only on Windows")
}

#[cfg(windows)]
fn process_name(pid: u32) -> Result<Option<String>> {
    let filter = format!("PID eq {pid}");
    let output = ProcessCommand::new("tasklist.exe")
        .args(["/FI", &filter, "/FO", "CSV", "/NH"])
        .output()
        .context("cannot inspect the existing VaMender engine process")?;
    if !output.status.success() {
        bail!("cannot inspect existing VaMender engine PID {pid}");
    }
    let listing = String::from_utf8_lossy(&output.stdout);
    if !listing.contains("\",\"") {
        return Ok(None);
    }
    let name = listing
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .trim_matches('"')
        .split("\",\"")
        .next()
        .unwrap_or("");
    Ok(Some(name.to_string()))
}

#[cfg(windows)]
fn remove_engine_lock(lock: &Path) -> Result<()> {
    if lock.exists() {
        fs::remove_file(lock)
            .with_context(|| format!("cannot clear stopped engine lock {}", lock.display()))?;
    }
    Ok(())
}

fn host_state(vam_root: &Path) -> PathBuf {
    vam_root
        .join("Saves")
        .join("PluginData")
        .join("VaMender")
        .join("Bridge")
}

fn is_vamender_host_process(name: &str) -> bool {
    name.eq_ignore_ascii_case("vamender.exe") || name.eq_ignore_ascii_case(HOST_EXECUTABLE_NAME)
}

fn engine_is_busy(state: &Path) -> Result<bool> {
    if state.join("request.json").is_file() {
        return Ok(true);
    }
    let status_path = state.join("status.txt");
    if !status_path.is_file() {
        return Ok(false);
    }
    let current = fs::read_to_string(&status_path)
        .with_context(|| format!("cannot read engine status {}", status_path.display()))?;
    Ok(current.trim_start().starts_with("RUNNING:"))
}

#[cfg(windows)]
fn stop_existing_host(vam_root: &Path) -> Result<()> {
    let state = host_state(vam_root);
    let lock = state.join("bridge.lock");
    if !lock.is_file() {
        return Ok(());
    }
    let pid_text = fs::read_to_string(&lock)
        .with_context(|| format!("cannot read existing engine lock {}", lock.display()))?;
    let pid = pid_text
        .trim()
        .parse::<u32>()
        .with_context(|| format!("invalid engine PID in {}", lock.display()))?;
    let Some(name) = process_name(pid)? else {
        return remove_engine_lock(&lock);
    };
    if !is_vamender_host_process(&name) {
        bail!("engine lock PID {pid} belongs to {name}; refusing to stop an unrelated process");
    }
    if engine_is_busy(&state)? {
        bail!(
            "VaMender is running or has a queued operation; wait for it to finish before installing or uninstalling"
        );
    }

    let shutdown = state.join(SHUTDOWN_FILE);
    fs::write(&shutdown, format!("stop requested for engine PID {pid}\n"))
        .with_context(|| format!("cannot request engine shutdown at {}", shutdown.display()))?;
    for _ in 0..600 {
        if process_name(pid)?.is_none() {
            let _ = fs::remove_file(&shutdown);
            return remove_engine_lock(&lock);
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    bail!(
        "VaMender engine PID {pid} did not stop cooperatively within 60 seconds; wait for any active operation, exit VaMender from its tray menu, and retry"
    )
}
#[cfg(not(windows))]
fn stop_existing_host(_vam_root: &Path) -> Result<()> {
    Ok(())
}

#[cfg(windows)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstalledHostConfiguration {
    addon_packages: PathBuf,
    backup: PathBuf,
    executable: PathBuf,
    #[serde(default)]
    host_executable: Option<PathBuf>,
    vam_root: PathBuf,
}

#[cfg(windows)]
fn installed_host_configuration(install_root: &Path) -> Result<Option<InstalledHostConfiguration>> {
    let path = install_root.join("host.json");
    if !path.is_file() {
        return Ok(None);
    }
    let value = serde_json::from_slice(
        &fs::read(&path).with_context(|| format!("cannot read {}", path.display()))?,
    )
    .with_context(|| format!("cannot parse {}", path.display()))?;
    Ok(Some(value))
}

#[cfg(windows)]
#[allow(dead_code)]
pub(super) fn installed_host_arguments() -> Result<BridgeArgs> {
    let install_root = local_app_data()?;
    let configuration = installed_host_configuration(&install_root)?
        .context("VaMender is not installed for this Windows user")?;
    Ok(BridgeArgs {
        root: configuration.addon_packages,
        backup: configuration.backup,
        state: None,
        out: None,
        poll_ms: 500,
        once: false,
    })
}

#[cfg(not(windows))]
#[allow(dead_code)]
pub(super) fn installed_host_arguments() -> Result<BridgeArgs> {
    bail!("the VaMender tray host is supported only on Windows")
}

fn configured_vam_root(install_root: &Path) -> Result<Option<PathBuf>> {
    let path = install_root.join("host.json");
    if !path.is_file() {
        return Ok(None);
    }
    let value: Value = serde_json::from_slice(
        &fs::read(&path).with_context(|| format!("cannot read {}", path.display()))?,
    )
    .with_context(|| format!("cannot parse {}", path.display()))?;
    Ok(value
        .get("vamRoot")
        .and_then(Value::as_str)
        .map(PathBuf::from))
}

#[cfg(windows)]
pub(super) fn restart_installed_host() -> Result<bool> {
    let install_root = local_app_data()?;
    let Some(configuration) = installed_host_configuration(&install_root)? else {
        return Ok(false);
    };
    let lock = configuration
        .vam_root
        .join("Saves")
        .join("PluginData")
        .join("VaMender")
        .join("Bridge")
        .join("bridge.lock");
    if let Ok(pid) = fs::read_to_string(&lock).and_then(|value| {
        value
            .trim()
            .parse::<u32>()
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }) {
        if process_name(pid)?.is_some_and(|name| is_vamender_host_process(&name)) {
            return Ok(true);
        }
    }
    if let Some(host_executable) = configuration.host_executable.filter(|path| path.is_file()) {
        start_host(&host_executable)?;
    } else {
        start_legacy_host(
            &configuration.executable,
            &configuration.addon_packages,
            &configuration.backup,
        )?;
    }
    Ok(true)
}

#[cfg(not(windows))]
pub(super) fn restart_installed_host() -> Result<bool> {
    Ok(false)
}

fn startup_command(executable: &Path) -> Result<String> {
    if executable.to_string_lossy().contains('"') {
        bail!("VaMender installation paths cannot contain a double quote");
    }
    Ok(format!("\"{}\"", executable.display()))
}

#[cfg(windows)]
pub(super) fn start_with_windows_enabled() -> bool {
    ProcessCommand::new("reg.exe")
        .args([
            "QUERY",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
            "/v",
            "VaMender",
        ])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(windows)]
pub(super) fn set_start_with_windows(enabled: bool) -> Result<()> {
    if enabled {
        let install_root = local_app_data()?;
        let configuration = installed_host_configuration(&install_root)?
            .context("VaMender is not installed for this Windows user")?;
        let host_executable = configuration
            .host_executable
            .filter(|path| path.is_file())
            .context("the installed VaMender background host is missing; run Setup to repair it")?;
        let command = startup_command(&host_executable)?;
        run_reg(&[
            "ADD",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
            "/v",
            "VaMender",
            "/t",
            "REG_SZ",
            "/d",
            &command,
            "/f",
        ])
    } else {
        let _ = run_reg(&[
            "DELETE",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
            "/v",
            "VaMender",
            "/f",
        ]);
        Ok(())
    }
}

const PLUGIN_CREATOR: &str = "AgenticCreator";
const PLUGIN_PACKAGE: &str = "VaMender";
const PLUGIN_REVISION: u32 = 2;
const PLUGIN_FILENAME: &str = "AgenticCreator.VaMender.2.var";

fn preserve_plugin(package: &Path, backup: &Path) -> Result<PathBuf> {
    let name = package.file_name().context("plugin VAR has no filename")?;
    let digest = sha256(package)?;
    let previous = backup
        .join("install-history")
        .join(format!("{digest}-{}", name.to_string_lossy()));
    fs::create_dir_all(previous.parent().context("invalid plugin backup path")?)?;
    fs::copy(package, &previous).with_context(|| {
        format!(
            "cannot preserve existing plugin {} as {}",
            package.display(),
            previous.display()
        )
    })?;
    if sha256(&previous)? != digest {
        bail!(
            "plugin backup checksum does not match source: {}",
            previous.display()
        );
    }
    Ok(previous)
}

fn older_plugin_revisions(packages: &Path) -> Result<Vec<PathBuf>> {
    let mut revisions = Vec::new();
    for entry in fs::read_dir(packages)
        .with_context(|| format!("cannot list plugin directory {}", packages.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(id) = package_file_id(&name) else {
            continue;
        };
        if id.creator == PLUGIN_CREATOR
            && id.package == PLUGIN_PACKAGE
            && id.version < PLUGIN_REVISION
        {
            revisions.push(entry.path());
        }
    }
    revisions.sort();
    Ok(revisions)
}

fn install_plugin(source: &Path, packages: &Path, backup: &Path) -> Result<()> {
    let source = fs::canonicalize(source)
        .with_context(|| format!("cannot resolve plugin VAR {}", source.display()))?;
    let name = source.file_name().context("plugin VAR has no filename")?;
    if name.to_string_lossy() != PLUGIN_FILENAME {
        bail!("expected {PLUGIN_FILENAME}, got {}", source.display());
    }
    let destination = packages.join(name);
    if destination.exists() {
        preserve_plugin(&destination, backup)?;
    }
    fs::copy(&source, &destination).with_context(|| {
        format!(
            "cannot install plugin {} as {}",
            source.display(),
            destination.display()
        )
    })?;
    if sha256(&source)? != sha256(&destination)? {
        bail!(
            "installed plugin checksum does not match source: {}",
            destination.display()
        );
    }

    for older in older_plugin_revisions(packages)? {
        preserve_plugin(&older, backup)?;
        fs::remove_file(&older).with_context(|| {
            format!(
                "cannot retire old VaMender plugin revision {} after backup",
                older.display()
            )
        })?;
    }
    Ok(())
}

pub(super) fn install_host(arguments: InstallHostArgs) -> Result<()> {
    let (vam_root, packages, backup) =
        validate_install_paths(&arguments.vam_root, &arguments.backup)?;
    let install_root = local_app_data()?;
    fs::create_dir_all(&install_root)
        .with_context(|| format!("cannot create {}", install_root.display()))?;
    let _ = run_schtasks(&["/End", "/TN", TASK_NAME]);
    let _ = run_schtasks(&["/Delete", "/F", "/TN", TASK_NAME]);
    stop_existing_host(&vam_root)?;
    let current = std::env::current_exe().context("cannot locate the VaMender executable")?;
    let installed = install_root.join("vamender.exe");
    if current != installed {
        fs::copy(&current, &installed).with_context(|| {
            format!(
                "cannot install engine {} as {}",
                current.display(),
                installed.display()
            )
        })?;
    }
    let host_source = current
        .parent()
        .context("VaMender executable has no parent folder")?
        .join(HOST_EXECUTABLE_NAME);
    if !host_source.is_file() {
        bail!(
            "cannot find {} next to {}; run the complete VaMender Setup package",
            HOST_EXECUTABLE_NAME,
            current.display()
        );
    }
    let installed_host = install_root.join(HOST_EXECUTABLE_NAME);
    if host_source != installed_host {
        fs::copy(&host_source, &installed_host).with_context(|| {
            format!(
                "cannot install background host {} as {}",
                host_source.display(),
                installed_host.display()
            )
        })?;
    }

    let configuration = json!({
        "addonPackages": packages,
        "backup": backup,
        "executable": installed,
        "hostExecutable": installed_host,
        "vamRoot": vam_root,
        "installedAtUnix": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    });
    fs::write(
        install_root.join("host.json"),
        serde_json::to_vec_pretty(&configuration)?,
    )
    .context("cannot write VaMender host configuration")?;

    if let Some(plugin) = arguments.plugin_var.as_deref() {
        install_plugin(plugin, &packages, &backup)?;
    }

    let command = startup_command(&installed_host)?;
    let _ = run_schtasks(&["/End", "/TN", TASK_NAME]);
    let _ = run_schtasks(&["/Delete", "/F", "/TN", TASK_NAME]);
    run_reg(&[
        "ADD",
        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
        "/v",
        "VaMender",
        "/t",
        "REG_SZ",
        "/d",
        &command,
        "/f",
    ])?;
    start_host(&installed_host)?;
    println!("VaMender automatic integration installed.");
    println!("VaM: {}", vam_root.display());
    println!("Background host: {}", installed_host.display());
    println!("Library: {}", packages.display());
    println!("Backup: {}", backup.display());
    println!("No PowerShell script or open console is required.");
    Ok(())
}

pub(super) fn stop_installed_host() -> Result<()> {
    let install_root = local_app_data()?;
    if let Some(vam_root) = configured_vam_root(&install_root)? {
        stop_existing_host(&vam_root)?;
    }
    println!("VaMender tray host stopped safely.");
    Ok(())
}

pub(super) fn uninstall_host(arguments: UninstallHostArgs) -> Result<()> {
    let install_root = local_app_data()?;
    if let Some(vam_root) = configured_vam_root(&install_root)? {
        stop_existing_host(&vam_root)?;
    }
    let _ = run_reg(&[
        "DELETE",
        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
        "/v",
        "VaMender",
        "/f",
    ]);
    let _ = run_schtasks(&["/End", "/TN", TASK_NAME]);
    let _ = run_schtasks(&["/Delete", "/F", "/TN", TASK_NAME]);
    if arguments.purge {
        let executable = install_root.join("vamender.exe");
        let host_executable = install_root.join(HOST_EXECUTABLE_NAME);
        let configuration = install_root.join("host.json");
        if configuration.exists() {
            fs::remove_file(&configuration)
                .with_context(|| format!("cannot remove {}", configuration.display()))?;
        }
        if executable.exists() && std::env::current_exe().ok().as_deref() != Some(&executable) {
            fs::remove_file(&executable)
                .with_context(|| format!("cannot remove {}", executable.display()))?;
        }
        if host_executable.exists()
            && std::env::current_exe().ok().as_deref() != Some(&host_executable)
        {
            fs::remove_file(&host_executable)
                .with_context(|| format!("cannot remove {}", host_executable.display()))?;
        }
    }
    println!(
        "VaMender automatic integration removed. Backups, reports, and the plugin VAR were retained."
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_configured_vam_root_for_safe_uninstall() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        fs::write(
            temporary.path().join("host.json"),
            br#"{"vamRoot":"D:\\VaM"}"#,
        )?;
        assert_eq!(
            configured_vam_root(temporary.path())?,
            Some(PathBuf::from(r"D:\VaM"))
        );
        Ok(())
    }

    #[test]
    fn quotes_background_host_startup_path_with_spaces() -> Result<()> {
        let command = startup_command(Path::new(r"C:\Users\Test User\VaMender\vamender-host.exe"))?;
        assert_eq!(
            command,
            "\"C:\\Users\\Test User\\VaMender\\vamender-host.exe\""
        );
        Ok(())
    }

    #[test]
    fn recognizes_current_and_legacy_host_process_names() {
        assert!(is_vamender_host_process("vamender-host.exe"));
        assert!(is_vamender_host_process("VaMender.exe"));
        assert!(!is_vamender_host_process("VaM.exe"));
    }

    #[test]
    fn rejects_backup_inside_addon_packages() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let root = temporary.path().join("VaM");
        fs::create_dir_all(root.join("AddonPackages").join("backup"))?;
        fs::write(root.join("VaM.exe"), b"test")?;
        let error = validate_install_paths(&root, &root.join("AddonPackages").join("backup"))
            .expect_err("nested backup must fail");
        assert!(error.to_string().contains("must be outside"));
        Ok(())
    }

    #[test]
    fn preserves_and_replaces_existing_current_plugin() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let source_root = temporary.path().join("release");
        let packages = temporary.path().join("AddonPackages");
        let backup = temporary.path().join("backup");
        fs::create_dir_all(&source_root)?;
        fs::create_dir_all(&packages)?;
        fs::create_dir_all(&backup)?;
        let name = "AgenticCreator.VaMender.2.var";
        let source = source_root.join(name);
        fs::write(&source, b"new")?;
        fs::write(packages.join(name), b"old")?;

        install_plugin(&source, &packages, &backup)?;

        assert_eq!(fs::read(packages.join(name))?, b"new");
        let old_hash = Sha256::digest(b"old");
        let old_hash = format!("{old_hash:x}");
        assert_eq!(
            fs::read(
                backup
                    .join("install-history")
                    .join(format!("{old_hash}-{name}")),
            )?,
            b"old"
        );
        Ok(())
    }

    #[test]
    fn retires_and_preserves_older_plugin_revisions() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let source_root = temporary.path().join("release");
        let packages = temporary.path().join("AddonPackages");
        let backup = temporary.path().join("backup");
        fs::create_dir_all(&source_root)?;
        fs::create_dir_all(&packages)?;
        fs::create_dir_all(&backup)?;

        let source = source_root.join(PLUGIN_FILENAME);
        let older = packages.join("AgenticCreator.VaMender.1.var");
        let newer = packages.join("AgenticCreator.VaMender.3.var");
        fs::write(&source, b"revision-two")?;
        fs::write(&older, b"revision-one")?;
        fs::write(&newer, b"revision-three")?;

        install_plugin(&source, &packages, &backup)?;

        assert_eq!(fs::read(packages.join(PLUGIN_FILENAME))?, b"revision-two");
        assert!(!older.exists());
        assert_eq!(fs::read(&newer)?, b"revision-three");
        let old_hash = format!("{:x}", Sha256::digest(b"revision-one"));
        assert_eq!(
            fs::read(
                backup
                    .join("install-history")
                    .join(format!("{old_hash}-AgenticCreator.VaMender.1.var")),
            )?,
            b"revision-one"
        );
        Ok(())
    }

    #[test]
    fn detects_active_or_queued_engine_work() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        assert!(!engine_is_busy(temporary.path())?);
        fs::write(
            temporary.path().join("status.txt"),
            b"RUNNING: repair request 1",
        )?;
        assert!(engine_is_busy(temporary.path())?);
        fs::write(
            temporary.path().join("status.txt"),
            b"COMPLETE: repair finished",
        )?;
        assert!(!engine_is_busy(temporary.path())?);
        fs::write(temporary.path().join("request.json"), b"{}")?;
        assert!(engine_is_busy(temporary.path())?);
        Ok(())
    }
}

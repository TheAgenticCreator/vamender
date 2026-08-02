// SPDX-License-Identifier: MIT

use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;

const REQUEST_FILE: &str = "request.json";
const STATUS_FILE: &str = "status.txt";
const RESPONSE_FILE: &str = "response.json";
const HEARTBEAT_FILE: &str = "heartbeat.txt";
const LOCK_FILE: &str = "bridge.lock";
pub(super) const SHUTDOWN_FILE: &str = "shutdown.request";

#[derive(Debug, Deserialize)]
struct BridgeRequest {
    id: String,
    operation: String,
    #[serde(default)]
    deep: bool,
    license: Option<String>,
}

#[derive(Serialize)]
struct BridgeResponse<'a> {
    id: &'a str,
    operation: &'a str,
    success: bool,
    message: &'a str,
    reports: String,
}

struct BridgeLock {
    path: PathBuf,
}

impl Drop for BridgeLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn acquire_lock(state: &Path) -> Result<BridgeLock> {
    let path = state.join(LOCK_FILE);
    let heartbeat_path = state.join(HEARTBEAT_FILE);
    if path.exists() {
        let fresh_heartbeat = heartbeat_path
            .metadata()
            .and_then(|metadata| metadata.modified())
            .and_then(|modified| modified.elapsed().map_err(io::Error::other))
            .map(|elapsed| elapsed <= Duration::from_secs(5))
            .unwrap_or(false);
        if fresh_heartbeat {
            bail!(
                "another VaMender engine is already monitoring {}",
                state.display()
            );
        }
        fs::remove_file(&path)?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    writeln!(file, "{}", std::process::id())?;
    Ok(BridgeLock { path })
}

fn bridge_state(arguments: &BridgeArgs) -> PathBuf {
    arguments.state.clone().unwrap_or_else(|| {
        arguments
            .root
            .parent()
            .unwrap_or(&arguments.root)
            .join("Saves")
            .join("PluginData")
            .join("VaMender")
            .join("Bridge")
    })
}

fn write_text(path: &Path, value: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut temporary = NamedTempFile::new_in(path.parent().unwrap_or_else(|| Path::new(".")))?;
    temporary.write_all(value.as_bytes())?;
    temporary.flush()?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    temporary
        .persist(path)
        .map_err(|error| error.error)
        .with_context(|| format!("cannot publish engine state file {}", path.display()))?;
    Ok(())
}

fn heartbeat(state: &Path) -> Result<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let path = state.join(HEARTBEAT_FILE);
    fs::write(&path, format!("VaMender engine ready at Unix time {now}"))
        .with_context(|| format!("cannot publish heartbeat {}", path.display()))
}

fn start_heartbeat_worker(
    state: PathBuf,
    poll_ms: u64,
    stop: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let interval = Duration::from_millis(poll_ms.max(250));
        while !stop.load(Ordering::Acquire) {
            if let Err(error) = heartbeat(&state) {
                eprintln!("VaMender engine heartbeat failed: {error}");
            }
            thread::sleep(interval);
        }
    })
}

fn status(state: &Path, value: &str) -> Result<()> {
    println!("{value}");
    write_text(&state.join(STATUS_FILE), value)
}

fn report_path(arguments: &BridgeArgs, state: &Path, request: &BridgeRequest) -> PathBuf {
    arguments
        .out
        .clone()
        .unwrap_or_else(|| state.join("reports"))
        .join(&request.id)
        .join(&request.operation)
}

fn process_request(
    arguments: &BridgeArgs,
    state: &Path,
    request: &BridgeRequest,
) -> Result<PathBuf> {
    let reports = report_path(arguments, state, request);
    match request.operation.as_str() {
        "check" => run_inspect(InspectArgs {
            root: arguments.root.clone(),
            out: Some(reports.clone()),
            deep: request.deep,
        })?,
        "plan" => run_plan(OptimizeArgs {
            root: arguments.root.clone(),
            out: Some(reports.clone()),
            vam_log: None,
        })?,
        "repair" => {
            run_repair(RepairArgs {
                root: arguments.root.clone(),
                out: Some(reports.clone()),
                apply: true,
                backup: Some(arguments.backup.clone()),
                license: request.license.clone(),
                non_interactive: true,
            })?;
        }
        "migrate" => {
            run_migrate(MigrationArgs {
                root: arguments.root.clone(),
                out: Some(reports.clone()),
                apply: true,
                backup: Some(arguments.backup.clone()),
            })?;
        }
        "run" => {
            run_all(RunArgs {
                root: arguments.root.clone(),
                backup: arguments.backup.clone(),
                out: Some(reports.clone()),
                license: request.license.clone(),
            })?;
        }
        "restore-last" => {
            run_restore(RestoreArgs {
                root: arguments.root.clone(),
                manifest: arguments.backup.join("manifest.jsonl"),
                overwrite: true,
                last: Some(1),
            })?;
        }
        "restore-all" => {
            run_restore(RestoreArgs {
                root: arguments.root.clone(),
                manifest: arguments.backup.join("manifest.jsonl"),
                overwrite: true,
                last: None,
            })?;
        }
        other => bail!("unknown engine operation: {other}"),
    }
    Ok(reports)
}

fn read_request(path: &Path) -> Result<BridgeRequest> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("cannot read engine request {}", path.display()))?;
    let request: BridgeRequest = serde_json::from_str(&text)
        .with_context(|| format!("invalid engine request {}", path.display()))?;
    if request.id.is_empty()
        || request.id.len() > 32
        || !request.id.bytes().all(|value| value.is_ascii_digit())
    {
        bail!("engine request ID must contain 1 to 32 ASCII digits");
    }
    Ok(request)
}

fn write_response(state: &Path, request: &BridgeRequest, result: &Result<PathBuf>) -> Result<()> {
    let (success, message, reports) = match result {
        Ok(reports) => (
            true,
            "VaMender operation completed. Review the generated reports.",
            reports.display().to_string(),
        ),
        Err(error) => (false, "VaMender operation failed.", error.to_string()),
    };
    let response = BridgeResponse {
        id: &request.id,
        operation: &request.operation,
        success,
        message,
        reports,
    };
    write_text(
        &state.join(RESPONSE_FILE),
        &serde_json::to_string_pretty(&response)?,
    )?;
    if success {
        status(
            state,
            &format!(
                "COMPLETE: {} finished. Review {}.",
                request.operation,
                result.as_ref().unwrap().display()
            ),
        )
    } else {
        status(
            state,
            &format!(
                "FAILED: {}. {}",
                request.operation,
                result.as_ref().unwrap_err()
            ),
        )
    }
}

fn monitor_requests(arguments: &BridgeArgs, state: &Path, stop: &AtomicBool) -> Result<()> {
    while !stop.load(Ordering::Acquire) {
        let request_path = state.join(REQUEST_FILE);
        if request_path.is_file() {
            match read_request(&request_path) {
                Ok(request) => {
                    fs::remove_file(&request_path)?;
                    status(
                        state,
                        &format!(
                            "RUNNING: VaMender {} request {}.",
                            request.operation, request.id
                        ),
                    )?;
                    let result = process_request(arguments, state, &request);
                    write_response(state, &request, &result)?;
                }
                Err(error) => {
                    fs::remove_file(&request_path)?;
                    status(state, &format!("FAILED: invalid request. {error}"))?;
                }
            }
        }
        if arguments.once {
            break;
        }
        thread::sleep(Duration::from_millis(arguments.poll_ms.max(100)));
    }
    Ok(())
}

pub(super) fn run_bridge_until(arguments: BridgeArgs, stop: Arc<AtomicBool>) -> Result<()> {
    if !arguments.root.is_dir() {
        bail!(
            "AddonPackages folder does not exist: {}",
            arguments.root.display()
        );
    }
    if arguments.backup.starts_with(&arguments.root) {
        bail!("engine backup directory must be outside AddonPackages");
    }
    fs::create_dir_all(&arguments.backup)?;
    let state = bridge_state(&arguments);
    fs::create_dir_all(&state)?;
    let _lock = acquire_lock(&state)?;
    let heartbeat_worker =
        start_heartbeat_worker(state.clone(), arguments.poll_ms, Arc::clone(&stop));
    status(
        &state,
        &format!(
            "READY: VaMender engine is monitoring {}. Library: {}. Backup: {}.",
            state.display(),
            arguments.root.display(),
            arguments.backup.display()
        ),
    )?;
    println!("VaMender engine is ready for in-VaM requests.");

    let result = monitor_requests(&arguments, &state, &stop);
    stop.store(true, Ordering::Release);
    let _ = heartbeat_worker.join();
    if result.is_ok() {
        status(&state, "STOPPED: VaMender engine exited cleanly.")?;
    }
    result
}

pub(super) fn run_bridge(arguments: BridgeArgs) -> Result<()> {
    run_bridge_until(arguments, Arc::new(AtomicBool::new(false)))
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_report_path_traversal_in_request_id() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let request = temporary.path().join("request.json");
        fs::write(
            &request,
            br#"{"id":"..\\..\\outside","operation":"check","deep":false}"#,
        )?;
        let error = read_request(&request).expect_err("path traversal ID must fail");
        assert!(error.to_string().contains("ASCII digits"));
        Ok(())
    }

    #[test]
    fn accepts_plugin_tick_request_id() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let request = temporary.path().join("request.json");
        fs::write(
            &request,
            br#"{"id":"639210177177945229","operation":"check","deep":false}"#,
        )?;
        assert_eq!(read_request(&request)?.id, "639210177177945229");
        Ok(())
    }
}

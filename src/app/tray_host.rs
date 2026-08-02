// SPDX-License-Identifier: MIT

use super::*;

#[cfg(windows)]
mod windows {
    use super::*;
    use std::mem::MaybeUninit;
    use std::os::windows::process::CommandExt;
    use std::process::{Command as ProcessCommand, Stdio};
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    };
    use std::thread;
    use std::time::Duration;
    use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
    use tray_icon::{Icon, TrayIconBuilder};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, MB_ICONERROR, MB_ICONINFORMATION, MB_OK, MSG, MessageBoxW, PM_REMOVE,
        PeekMessageW, TranslateMessage,
    };

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    struct MenuIds {
        launch_vam: MenuId,
        reports: MenuId,
        backup: MenuId,
        startup: MenuId,
        about: MenuId,
        exit: MenuId,
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn message(title: &str, body: &str, error: bool) {
        let title = wide(title);
        let body = wide(body);
        let icon = if error {
            MB_ICONERROR
        } else {
            MB_ICONINFORMATION
        };
        unsafe {
            MessageBoxW(
                std::ptr::null_mut(),
                body.as_ptr(),
                title.as_ptr(),
                MB_OK | icon,
            );
        }
    }

    fn pump_windows_messages() {
        let mut message = MaybeUninit::<MSG>::zeroed();
        unsafe {
            while PeekMessageW(message.as_mut_ptr(), std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(message.as_ptr());
                DispatchMessageW(message.as_ptr());
            }
        }
    }

    fn open_folder(path: &Path) -> Result<()> {
        fs::create_dir_all(path)
            .with_context(|| format!("cannot create folder {}", path.display()))?;
        ProcessCommand::new("explorer.exe")
            .arg(path)
            .spawn()
            .with_context(|| format!("cannot open folder {}", path.display()))?;
        Ok(())
    }

    fn vam_is_running() -> bool {
        ProcessCommand::new("tasklist.exe")
            .args(["/FI", "IMAGENAME eq VaM.exe", "/FO", "CSV", "/NH"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map(|output| {
                output.status.success()
                    && String::from_utf8_lossy(&output.stdout)
                        .to_ascii_lowercase()
                        .contains("vam.exe")
            })
            .unwrap_or(false)
    }

    fn launch_vam(root: &Path) -> Result<()> {
        if vam_is_running() {
            message("VaMender", "Virt-a-Mate is already running.", false);
            return Ok(());
        }
        let executable = root.parent().unwrap_or(root).join("VaM.exe");
        if !executable.is_file() {
            bail!("VaM.exe was not found at {}", executable.display());
        }
        ProcessCommand::new(&executable)
            .current_dir(
                executable
                    .parent()
                    .context("VaM.exe has no parent folder")?,
            )
            .stdin(Stdio::null())
            .spawn()
            .with_context(|| format!("cannot launch {}", executable.display()))?;
        Ok(())
    }

    fn state_folder(arguments: &BridgeArgs) -> PathBuf {
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

    fn report_folder(arguments: &BridgeArgs) -> PathBuf {
        arguments
            .out
            .clone()
            .unwrap_or_else(|| state_folder(arguments).join("reports"))
    }

    fn build_menu() -> Result<(Menu, MenuIds, CheckMenuItem, MenuItem)> {
        let menu = Menu::new();
        let status = MenuItem::with_id(
            "status",
            format!("VaMender {} — engine ready", env!("CARGO_PKG_VERSION")),
            false,
            None,
        );
        let launch_vam = MenuItem::with_id("launch-vam", "Launch Virt-a-Mate", true, None);
        let reports = MenuItem::with_id("reports", "Open reports folder", true, None);
        let backup = MenuItem::with_id("backup", "Open backup folder", true, None);
        let startup = CheckMenuItem::with_id(
            "startup",
            "Start with Windows",
            true,
            start_with_windows_enabled(),
            None,
        );
        let about = MenuItem::with_id("about", "About VaMender", true, None);
        let exit = MenuItem::with_id("exit", "Exit VaMender", true, None);
        let first_separator = PredefinedMenuItem::separator();
        let second_separator = PredefinedMenuItem::separator();
        menu.append_items(&[
            &status,
            &first_separator,
            &launch_vam,
            &reports,
            &backup,
            &startup,
            &second_separator,
            &about,
            &exit,
        ])?;
        let ids = MenuIds {
            launch_vam: launch_vam.id().clone(),
            reports: reports.id().clone(),
            backup: backup.id().clone(),
            startup: startup.id().clone(),
            about: about.id().clone(),
            exit: exit.id().clone(),
        };
        Ok((menu, ids, startup, status))
    }

    fn handle_action(
        id: &MenuId,
        ids: &MenuIds,
        startup: &CheckMenuItem,
        arguments: &BridgeArgs,
        executable: &Path,
    ) -> Result<bool> {
        if id == &ids.launch_vam {
            launch_vam(&arguments.root)?;
        } else if id == &ids.reports {
            open_folder(&report_folder(arguments))?;
        } else if id == &ids.backup {
            open_folder(&arguments.backup)?;
        } else if id == &ids.startup {
            let requested = startup.is_checked();
            if let Err(error) =
                set_start_with_windows(requested, executable, &arguments.root, &arguments.backup)
            {
                startup.set_checked(!requested);
                return Err(error);
            }
        } else if id == &ids.about {
            message(
                "About VaMender",
                &format!(
                    "VaMender {}\n\nBackup-first VAR repair and dependency cleanup for Virt-a-Mate 1.22.0.12.\n\nVaMender is provided AS IS. Keep an independent, tested backup; you are responsible for your data, licenses, and every applied operation.",
                    env!("CARGO_PKG_VERSION")
                ),
                false,
            );
        } else if id == &ids.exit {
            return Ok(true);
        }
        Ok(false)
    }

    pub(super) fn run(arguments: BridgeArgs) -> Result<()> {
        let executable = std::env::current_exe().context("cannot locate VaMender executable")?;
        let shutdown_request = state_folder(&arguments).join(SHUTDOWN_FILE);
        match fs::remove_file(&shutdown_request) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| {
                    format!(
                        "cannot clear stale shutdown request {}",
                        shutdown_request.display()
                    )
                });
            }
        }
        let engine_arguments = arguments.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let engine_stop = Arc::clone(&stop);
        let (engine_sender, engine_receiver) = mpsc::sync_channel(1);
        let engine = thread::Builder::new()
            .name("vamender-engine".to_string())
            .stack_size(COMMAND_STACK_BYTES)
            .spawn(move || {
                let result = run_bridge_until(engine_arguments, engine_stop);
                let _ = engine_sender.send(result);
            })
            .context("cannot start VaMender engine worker")?;

        let (menu, ids, startup, status) = build_menu()?;
        let icon = Icon::from_resource(1, Some((32, 32)))
            .context("cannot load the VaMender executable icon")?;
        let _tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_icon(icon)
            .with_tooltip(format!(
                "VaMender {} — engine ready",
                env!("CARGO_PKG_VERSION")
            ))
            .build()
            .context("cannot create the VaMender notification-area icon")?;

        let mut exit_requested = false;
        while !exit_requested {
            if shutdown_request.is_file() {
                fs::remove_file(&shutdown_request).with_context(|| {
                    format!(
                        "cannot consume shutdown request {}",
                        shutdown_request.display()
                    )
                })?;
                exit_requested = true;
                continue;
            }
            pump_windows_messages();
            while let Ok(event) = MenuEvent::receiver().try_recv() {
                match handle_action(event.id(), &ids, &startup, &arguments, &executable) {
                    Ok(exit) => exit_requested |= exit,
                    Err(error) => message("VaMender", &error.to_string(), true),
                }
            }
            match engine_receiver.try_recv() {
                Ok(result) => {
                    result?;
                    bail!("VaMender engine stopped unexpectedly")
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    bail!("VaMender engine worker disconnected unexpectedly")
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
            thread::sleep(Duration::from_millis(50));
        }

        status.set_text("VaMender — stopping safely...");
        stop.store(true, Ordering::Release);
        while !engine.is_finished() {
            pump_windows_messages();
            thread::sleep(Duration::from_millis(50));
        }
        engine
            .join()
            .map_err(|_| anyhow::anyhow!("VaMender engine worker panicked"))?;
        match engine_receiver.try_recv() {
            Ok(result) => result,
            Err(_) => bail!("VaMender engine stopped without reporting its result"),
        }
    }
}

#[cfg(windows)]
pub(super) fn run_tray_host(arguments: BridgeArgs) -> Result<()> {
    windows::run(arguments)
}

#[cfg(not(windows))]
pub(super) fn run_tray_host(_arguments: BridgeArgs) -> Result<()> {
    bail!("the VaMender tray host is supported only on Windows")
}

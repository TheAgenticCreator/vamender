// SPDX-License-Identifier: MIT

#![cfg_attr(windows, windows_subsystem = "windows")]

#[path = "app/mod.rs"]
#[allow(dead_code)]
mod app;

fn main() -> anyhow::Result<()> {
    app::run_installed_tray_host()
}

// SPDX-License-Identifier: MIT

use super::*;

#[derive(Parser)]
#[command(
    name = "vamender",
    version,
    about = "Backup-first repair and dependency cleanup for VaM AddonPackages",
    after_help = "SAFETY: Never operate on your only copy of AddonPackages. Keep an independent, tested full backup. VaMender is provided AS IS, without warranty; you are responsible for backups, licensing, and the results of every operation."
)]
pub(super) struct Cli {
    #[command(subcommand)]
    pub(super) command: Command,
}

#[derive(Subcommand)]
pub(super) enum Command {
    /// Read-only inventory and dependency check. Use --deep for a full CRC pass.
    #[command(alias = "inspect")]
    Check(InspectArgs),
    /// Read-only VaM-log-aware cleanup plan. No VAR is changed.
    #[command(alias = "optimize", alias = "prune-plan")]
    Plan(OptimizeArgs),
    /// Plan or apply safe filename, metadata, and archive repair.
    Repair(RepairArgs),
    /// Plan or apply conservative old-version cleanup.
    Migrate(MigrationArgs),
    /// Apply repairs, safe relinks, dependency-closure archiving, and conservative migration without review gates.
    Run(RunArgs),
    /// Restore backed-up VARs from a manifest created by this tool.
    Restore(RestoreArgs),
    /// Create a local, review-first diagnostic bundle for GitHub support.
    #[command(name = "support-report", alias = "support", alias = "diagnostics")]
    SupportReport(SupportReportArgs),
    /// Install the automatic VaM integration without PowerShell.
    #[command(hide = true)]
    InstallHost(InstallHostArgs),
    /// Remove the automatic VaM integration host.
    #[command(hide = true)]
    UninstallHost(UninstallHostArgs),
    /// Stop the installed tray host cooperatively without changing startup settings.
    #[command(name = "stop-host", hide = true)]
    StopHost,
    /// Internal tray-host mode used by automatic per-user startup.
    #[command(hide = true)]
    Host(BridgeArgs),
    /// Internal headless engine mode used by automated validation.
    #[command(hide = true)]
    Bridge(BridgeArgs),
}

#[derive(Args, Clone)]
pub(super) struct InspectArgs {
    /// VaM AddonPackages folder
    pub(super) root: PathBuf,
    /// Report folder (default: <root parent>/VaMVarReports)
    #[arg(long)]
    pub(super) out: Option<PathBuf>,
    /// CRC-read all archive members. Slower, but detects corrupt binary assets.
    #[arg(long)]
    pub(super) deep: bool,
}

#[derive(Args, Clone)]
pub(super) struct RepairArgs {
    /// VaM AddonPackages folder
    pub(super) root: PathBuf,
    #[arg(long)]
    pub(super) out: Option<PathBuf>,
    /// Required to modify live VARs. Without this flag, the command is a dry run.
    #[arg(long)]
    pub(super) apply: bool,
    /// Required durable backup directory when using --apply.
    #[arg(long)]
    pub(super) backup: Option<PathBuf>,
    /// Rebuild missing/invalid meta.json files using this explicit license.
    #[arg(long)]
    pub(super) license: Option<String>,
    /// Do not prompt for missing licenses. Such VARs remain unchanged unless --license is provided.
    #[arg(long)]
    pub(super) non_interactive: bool,
}

#[derive(Args, Clone)]
pub(super) struct MigrationArgs {
    /// VaM AddonPackages folder
    pub(super) root: PathBuf,
    #[arg(long)]
    pub(super) out: Option<PathBuf>,
    #[arg(long)]
    pub(super) apply: bool,
    #[arg(long)]
    pub(super) backup: Option<PathBuf>,
}

#[derive(Args, Clone)]
pub(super) struct OptimizeArgs {
    /// VaM AddonPackages folder
    pub(super) root: PathBuf,
    /// Report folder (default: <root parent>/VaMVarReports)
    #[arg(long)]
    pub(super) out: Option<PathBuf>,
    /// VaM output_log.txt. If omitted, the standard current-user log is detected automatically.
    #[arg(long)]
    pub(super) vam_log: Option<PathBuf>,
}

#[derive(Args, Clone)]
pub(super) struct RunArgs {
    /// VaM AddonPackages folder
    pub(super) root: PathBuf,
    /// Required durable backup directory for every rewrite/archive.
    #[arg(long)]
    pub(super) backup: PathBuf,
    /// Report folder (default: <root parent>/VaMVarReports)
    #[arg(long)]
    pub(super) out: Option<PathBuf>,
    /// Explicit license for rebuilding every missing/invalid meta.json. Without
    /// this, unknown-license VARs are safely left unchanged.
    #[arg(long)]
    pub(super) license: Option<String>,
}

#[derive(Args, Clone)]
pub(super) struct RestoreArgs {
    /// VaM AddonPackages folder
    pub(super) root: PathBuf,
    /// manifest.jsonl created in the backup folder
    pub(super) manifest: PathBuf,
    /// Replace a currently installed VAR after saving it under restore-conflicts
    #[arg(long)]
    pub(super) overwrite: bool,
    /// Restore only this many most-recent manifest records
    #[arg(long)]
    pub(super) last: Option<usize>,
}

#[derive(Args, Clone)]
pub(super) struct SupportReportArgs {
    /// VaM AddonPackages folder
    pub(super) root: PathBuf,
    /// Support report folder (default: <root parent>/VaMVarReports/support)
    #[arg(long)]
    pub(super) out: Option<PathBuf>,
    /// VaM output_log.txt. If omitted, the standard current-user log is detected automatically.
    #[arg(long)]
    pub(super) vam_log: Option<PathBuf>,
    /// CRC-read every archive member while collecting diagnostics.
    #[arg(long)]
    pub(super) deep: bool,
    /// Include every installed package ID/filename. This can reveal private content names.
    #[arg(long)]
    pub(super) include_var_list: bool,
    /// Open the GitHub support issue form after writing the local bundle. No files are uploaded.
    #[arg(long, requires = "confirm_reviewed")]
    pub(super) open_github: bool,
    /// Confirm that you reviewed the bundle and consent to opening GitHub. No diagnostic data is transmitted automatically.
    #[arg(long)]
    pub(super) confirm_reviewed: bool,
}

#[derive(Args, Clone)]
pub(super) struct BridgeArgs {
    /// VaM AddonPackages folder
    pub(super) root: PathBuf,
    /// Durable backup directory used by repair, migration, run, and restore
    #[arg(long)]
    pub(super) backup: PathBuf,
    /// Bridge state folder (default: <VaM>/Saves/PluginData/VaMender/Bridge)
    #[arg(long)]
    pub(super) state: Option<PathBuf>,
    /// Base report folder (default: <state>/reports)
    #[arg(long)]
    pub(super) out: Option<PathBuf>,
    /// Poll interval in milliseconds
    #[arg(long, default_value_t = 500)]
    pub(super) poll_ms: u64,
    /// Process at most one poll cycle; intended for automated validation
    #[arg(long, hide = true)]
    pub(super) once: bool,
}
#[derive(Args, Clone)]
pub(super) struct InstallHostArgs {
    /// VaM root folder containing VaM.exe and AddonPackages
    pub(super) vam_root: PathBuf,
    /// Durable backup directory outside AddonPackages
    #[arg(long)]
    pub(super) backup: PathBuf,
    /// Optional AgenticCreator.VaMender.2.var to install
    #[arg(long)]
    pub(super) plugin_var: Option<PathBuf>,
}

#[derive(Args, Clone)]
pub(super) struct UninstallHostArgs {
    /// Also remove the installed engine executable and configuration
    #[arg(long)]
    pub(super) purge: bool,
}

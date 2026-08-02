# SPDX-License-Identifier: MIT
# VaMender -- conservative, review-gated cleanup runbook.
# Run this script from PowerShell. It never downloads, repairs, or migrates
# until you type A at the corresponding review gate; C (or Enter) cancels.

[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string]$AddonPackages,

    [Parameter(Mandatory)]
    [string]$Backup,

    [string]$Reports,

    [string]$Tool = (Join-Path $PSScriptRoot "target\release\vamender.exe"),

    # Use after a migration-only failure. It creates a fresh migration plan and
    # never repeats already-applied metadata repairs.
    [switch]$ResumeMigration
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$projectRoot = $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($projectRoot)) {
    $projectRoot = (Get-Location).Path
}

if ([string]::IsNullOrWhiteSpace($Reports)) {
    $runId = Get-Date -Format "yyyyMMdd-HHmmss"
    $Reports = Join-Path $projectRoot "reports\\reviewed-cleanup-$runId"
}

function Invoke-VarManager {
    param([Parameter(Mandatory)][string[]]$ToolArgs)

    & $script:Tool @ToolArgs
    if ($LASTEXITCODE -ne 0) {
        throw "VaMender failed with exit code $LASTEXITCODE. No later stage was started."
    }
}

function Show-PlanAndConfirm {
    param(
        [Parameter(Mandatory)][string]$Stage,
        [Parameter(Mandatory)][string]$ReportDirectory,
        [Parameter(Mandatory)][string]$Prompt
    )

    Write-Host "`n===== $Stage =====" -ForegroundColor Cyan
    foreach ($name in @("actions_taken.txt", "actions_required.txt")) {
        $path = Join-Path $ReportDirectory $name
        if (Test-Path -LiteralPath $path -PathType Leaf) {
            Write-Host "`n--- $name ---" -ForegroundColor Yellow
            Get-Content -LiteralPath $path
        }
    }

    $missing = Join-Path $ReportDirectory "missing_dependencies.txt"
    if (Test-Path -LiteralPath $missing -PathType Leaf) {
        $items = @(Get-Content -LiteralPath $missing | Where-Object { $_.Trim() })
        if ($items.Count -gt 0) {
            Write-Host "`nUnresolved dependencies: $($items.Count). See $missing" -ForegroundColor Yellow
        }
    }

    $answer = Read-Host "$Prompt Type A to accept, or C to cancel [C]"
    if ($answer -ine "A") {
        Write-Host "Cancelled. Nothing from the pending stage was applied." -ForegroundColor Yellow
        exit 0
    }
}

if (-not (Test-Path -LiteralPath $Tool -PathType Leaf)) {
    throw "VaMender executable not found: $Tool. Download the project-published beta release or follow the README build instructions, then pass its path with -Tool."
}
if (-not (Test-Path -LiteralPath $AddonPackages -PathType Container)) {
    throw "AddonPackages folder not found: $AddonPackages"
}

New-Item -ItemType Directory -Force -Path $Backup, $Reports | Out-Null
$vamPath = (Resolve-Path -LiteralPath $AddonPackages).Path.TrimEnd('\\')
$backupPath = (Resolve-Path -LiteralPath $Backup).Path.TrimEnd('\\')
if ($backupPath.Equals($vamPath, [StringComparison]::OrdinalIgnoreCase) -or
    $backupPath.StartsWith($vamPath + "\\", [StringComparison]::OrdinalIgnoreCase)) {
    throw "Backup directory must be outside AddonPackages: $backupPath"
}

if (-not $ResumeMigration) {
    # 1. Read-only inventory. It uses the quick parallel scan; no VAR is changed.
    $check = Join-Path $Reports "01-check"
    Invoke-VarManager -ToolArgs @("check", $AddonPackages, "--out", $check)

    # 2. Read VaM's fresh package log when available and build the authoritative
    # no-change cleanup/closure plan. If the log predates a VAR change, its report
    # tells you to use VaM Package Manager > Rescan Packages, then
    # rerun this script. VaM has no supported headless package-rescan command.
    $plan = Join-Path $Reports "02-plan"
    Invoke-VarManager -ToolArgs @("plan", $AddonPackages, "--out", $plan)
    Write-Host "Read-only VaM reconciliation plan: $plan" -ForegroundColor Yellow

    # 3. Recalculate a repair plan after any packages you acquired through VaM
    # or the Hub yourself. Review it before metadata changes. No unknown license
    # is guessed.
    $repairPlan = Join-Path $Reports "03-repair-plan"
    Invoke-VarManager -ToolArgs @("repair", $AddonPackages, "--out", $repairPlan)
    Show-PlanAndConfirm -Stage "Metadata repair plan" -ReportDirectory $repairPlan -Prompt "Apply this metadata repair plan?"

    # If a missing/invalid meta.json needs rebuilding, choose its real license at
    # the program prompt. Type A there to leave this and all remaining unknown-
    # license packages unchanged; valid meta.json files are still synchronized.
    Assert-VaMClosed
    $repairApplied = Join-Path $Reports "04-repair-applied"
    Invoke-VarManager -ToolArgs @("repair", $AddonPackages, "--apply", "--backup", $Backup, "--out", $repairApplied)
} else {
    Write-Host "ResumeMigration selected: skipping inspection, optimization, and metadata repair stages." -ForegroundColor Yellow
}

# 4. Plan safe migration to the newest healthy local package versions. The
# default deliberately excludes scripts and changed Custom/Saves resources.
$migratePlan = Join-Path $Reports "05-migrate-plan"
Invoke-VarManager -ToolArgs @("migrate", $AddonPackages, "--out", $migratePlan)
Show-PlanAndConfirm -Stage "Old-version migration plan" -ReportDirectory $migratePlan -Prompt "Apply only these safe old-version migrations?"

Assert-VaMClosed
$migrateApplied = Join-Path $Reports "06-migrate-applied"
Invoke-VarManager -ToolArgs @("migrate", $AddonPackages, "--apply", "--backup", $Backup, "--out", $migrateApplied)

# 5. Final full CRC validation. This is intentionally last because it checks
# every untouched archive as well as the rewritten ones and can take a while.
$verify = Join-Path $Reports "07-final-deep-verify"
Invoke-VarManager -ToolArgs @("check", $AddonPackages, "--deep", "--out", $verify)

Write-Host "`nCleanup completed." -ForegroundColor Green
Write-Host "Final reports: $verify"
Write-Host "VaM reconciliation plan: $(Join-Path $Reports '02-plan')"
Write-Host "Backup manifest: $(Join-Path $Backup 'manifest.jsonl')"

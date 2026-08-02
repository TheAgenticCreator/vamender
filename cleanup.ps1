# SPDX-License-Identifier: MIT

[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string]$AddonPackages,

    [Parameter(Mandatory)]
    [string]$Backup,

    [string]$Reports,

    [string]$Tool = (Join-Path $PSScriptRoot "target\release\vamender.exe")
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($Reports)) {
    $runId = Get-Date -Format "yyyyMMdd-HHmmss"
    $Reports = Join-Path $PSScriptRoot "reports\automatic-cleanup-$runId"
}

if (-not (Test-Path -LiteralPath $Tool -PathType Leaf)) {
    throw "VaMender executable not found: $Tool. Download the project-published beta release or follow the README build instructions, then pass its path with -Tool."
}

& $Tool run $AddonPackages --backup $Backup --out $Reports
if ($LASTEXITCODE -ne 0) {
    throw "VaMender stopped safely; read $Reports\actions_required.txt"
}

Write-Host "VaMender cleanup complete. Reports: $Reports" -ForegroundColor Green

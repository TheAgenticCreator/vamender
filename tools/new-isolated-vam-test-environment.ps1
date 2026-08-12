# SPDX-License-Identifier: MIT

[CmdletBinding()]
param(
    [string]$SourceRoot = "C:\Users\trist\_\VAM\VaM",
    [string]$DestinationRoot = "C:\Users\trist\_\VAM\VaMender-ReleaseTest",
    [switch]$Reset
)

$ErrorActionPreference = "Stop"
$markerName = "VAMENDER-ISOLATED-TEST-ENVIRONMENT.txt"

function Invoke-Robocopy {
    param(
        [Parameter(Mandatory = $true)] [string]$Source,
        [Parameter(Mandatory = $true)] [string]$Destination,
        [string[]]$Files = @("*.*"),
        [switch]$Recursive
    )

    $arguments = @($Source, $Destination) + $Files + @("/COPY:DAT", "/DCOPY:DAT", "/R:2", "/W:1", "/NJH", "/NJS", "/NP")
    if ($Recursive) {
        $arguments += "/E"
    } else {
        $arguments += "/LEV:1"
    }
    & robocopy.exe @arguments
    if ($LASTEXITCODE -gt 7) {
        throw "robocopy failed with exit code $LASTEXITCODE while copying $Source"
    }
}

$source = [IO.Path]::GetFullPath($SourceRoot).TrimEnd('\')
$destination = [IO.Path]::GetFullPath($DestinationRoot).TrimEnd('\')
if ($source -eq $destination) {
    throw "The isolated test destination must not be the source VaM installation"
}
if (-not (Test-Path -LiteralPath (Join-Path $source "VaM.exe") -PathType Leaf)) {
    throw "Source VaM.exe is missing: $source"
}
foreach ($directory in @("VaM_Data", "Mono")) {
    if (-not (Test-Path -LiteralPath (Join-Path $source $directory) -PathType Container)) {
        throw "Source VaM runtime directory is missing: $directory"
    }
}

$marker = Join-Path $destination $markerName
if (Test-Path -LiteralPath $destination -PathType Container) {
    if (-not $Reset) {
        throw "Destination already exists. Re-run with -Reset only for a prior isolated VaMender test environment: $destination"
    }
    if (-not (Test-Path -LiteralPath $marker -PathType Leaf)) {
        throw "Refusing to reset an unmarked destination: $destination"
    }
    Remove-Item -LiteralPath $destination -Recurse -Force
}

New-Item -ItemType Directory -Force -Path $destination | Out-Null
$runtimeFiles = @(
    "Baseline Benchmark.bat",
    "browse_sites.json",
    "ClothSim Benchmark.bat",
    "config",
    "CPU Benchmark.bat",
    "CPU High Physics Benchmark.bat",
    "Crypt Benchmark.bat",
    "GPU Benchmark.bat",
    "Hair Benchmark.bat",
    "HairRender Benchmark.bat",
    "HairSim Benchmark.bat",
    "MHLab.PATCH.dll",
    "UnityCrashHandler64.exe",
    "UnityPlayer.dll",
    "VaM (Config).bat",
    "VaM (Desktop Mode).bat",
    "VaM (OpenVR).bat",
    "VaM.exe",
    "VaM_EULA.html",
    "VaM_Updater.exe",
    "version",
    "vrmanifest",
    "whitelist_domains.json",
    "WinPixEventRuntime.dll"
)
Invoke-Robocopy -Source $source -Destination $destination -Files $runtimeFiles
foreach ($directory in @("VaM_Data", "Mono", "Assets")) {
    $sourceDirectory = Join-Path $source $directory
    if (Test-Path -LiteralPath $sourceDirectory -PathType Container) {
        Invoke-Robocopy -Source $sourceDirectory -Destination (Join-Path $destination $directory) -Recursive
    }
}
foreach ($directory in @("AddonPackages", "AddonPackagesUserPrefs", "Saves")) {
    New-Item -ItemType Directory -Force -Path (Join-Path $destination $directory) | Out-Null
}

@(
    "VaMender isolated release-test environment",
    "Created: $([DateTime]::UtcNow.ToString('o'))",
    "Runtime source: $source",
    "User AddonPackages, Custom, Saves, Cache, BrowserProfile, Downloads, Keys, logs, and preferences were not copied.",
    "This directory may be reset only by tools/new-isolated-vam-test-environment.ps1 -Reset."
) | Set-Content -LiteralPath $marker

[pscustomobject]@{
    source_root = $source
    destination_root = $destination
    marker = $marker
    vam_exe = Join-Path $destination "VaM.exe"
    addon_packages = Join-Path $destination "AddonPackages"
} | ConvertTo-Json

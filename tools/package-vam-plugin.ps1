# SPDX-License-Identifier: MIT

[CmdletBinding()]
param(
    [string]$OutputPath = ".\dist\AgenticCreator.VaMender.1.var",
    [string]$PluginAssemblyPath =
        ".\vam-plugin\Custom\Scripts\AgenticCreator\VaMender\VaMender.dll"
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
$pluginRoot = Join-Path $projectRoot "vam-plugin"
$metaPath = Join-Path $pluginRoot "meta.json"
$scriptRoot = Join-Path $pluginRoot "Custom"
$resolvedPluginAssembly = [IO.Path]::GetFullPath(
    (Join-Path $projectRoot $PluginAssemblyPath)
)

if (-not (Test-Path -LiteralPath $metaPath -PathType Leaf)) {
    throw "Plugin meta.json is missing: $metaPath"
}
if (-not (Test-Path -LiteralPath $scriptRoot -PathType Container)) {
    throw "Plugin Custom folder is missing: $scriptRoot"
}
if (-not (Test-Path -LiteralPath $resolvedPluginAssembly -PathType Leaf)) {
    throw "Precompiled VaM plugin assembly is missing: $resolvedPluginAssembly"
}
$pluginAssemblyHash = (
    Get-FileHash -LiteralPath $resolvedPluginAssembly -Algorithm SHA256
).Hash.ToLowerInvariant()

$meta = Get-Content -LiteralPath $metaPath -Raw | ConvertFrom-Json
if ($meta.creatorName -ne "AgenticCreator" -or $meta.packageName -ne "VaMender") {
    throw "Plugin metadata must identify AgenticCreator.VaMender"
}
if ($meta.licenseType -ne "CC BY") {
    throw "Plugin metadata must use VaM license type CC BY"
}
if ($meta.programVersion -ne "1.22.0.13") {
    throw "Plugin metadata must target verified VaM version 1.22.0.13"
}

$cslistPath = Join-Path $scriptRoot "Scripts\AgenticCreator\VaMender\VaMender.cslist"
$cslistRoot = Split-Path -Parent $cslistPath
$listedSources = Get-Content -LiteralPath $cslistPath |
    Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
foreach ($source in $listedSources) {
    $resolved = Join-Path $cslistRoot $source
    if (-not (Test-Path -LiteralPath $resolved -PathType Leaf)) {
        throw "VaMender.cslist references a missing source: $source"
    }
}

$actualSources = Get-ChildItem -LiteralPath (Join-Path $cslistRoot "src") -File -Filter "*.cs"
if ($actualSources.Count -ne $listedSources.Count) {
    throw "VaMender.cslist must include every plugin source exactly once"
}

$legacyUnsafeSources = foreach ($sourceFile in $actualSources) {
    $sourceText = Get-Content -LiteralPath $sourceFile.FullName -Raw
    $usesUnsafePathApi = (
        $sourceFile.Name -ne "PluginPath.cs" -and
        $sourceText -match "\bPath\s*\.\s*Combine\s*\("
    )
    if ($usesUnsafePathApi -or
        $sourceText -match "\bSystem\s*\.\s*(IO|Reflection)\b" -or
        $sourceText -match "\bMVR\s*\.\s*FileManagement\b" -or
        $sourceText -match "\bSystem\s*\.\s*Diagnostics\s*\.\s*Process\b" -or
        $sourceText -match "\bApplication\s*\.\s*OpenURL\s*\(" -or
        $sourceText -match "\bVersion\s*\.\s*TryParse\s*\(" -or
        $sourceText -match "\bparams\s+" -or
        $sourceText -match "\bdelegate\s*(\(|\{)") {
        $sourceFile.Name
    }
}
if ($legacyUnsafeSources) {
    throw (
        "VaM plugin source uses APIs absent or unsafe in VaM's legacy .NET " +
        "profile: " + ($legacyUnsafeSources -join ", ") +
        ". Use fixed PluginPath.Combine overloads and Version constructors; " +
        "do not reference VaM-prohibited System.IO/System.Reflection or " +
        "MVR.FileManagement, Process, Application.OpenURL, params " +
        "arrays, or anonymous methods in VaM plugin source."
    )
}

$resolvedOutput = if ([IO.Path]::IsPathRooted($OutputPath)) {
    [IO.Path]::GetFullPath($OutputPath)
} else {
    [IO.Path]::GetFullPath((Join-Path $projectRoot $OutputPath))
}
$outputDirectory = Split-Path -Parent $resolvedOutput
New-Item -ItemType Directory -Force -Path $outputDirectory | Out-Null
if (Test-Path -LiteralPath $resolvedOutput) {
    Remove-Item -LiteralPath $resolvedOutput
}

Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$archive = [IO.Compression.ZipFile]::Open(
    $resolvedOutput,
    [IO.Compression.ZipArchiveMode]::Create
)
$timestamp = [DateTimeOffset]::new(2026, 1, 1, 0, 0, 0, [TimeSpan]::Zero)
try {
    $files = @(
        Get-Item -LiteralPath $metaPath
        Get-Item -LiteralPath $resolvedPluginAssembly
    )
    $files = $files | Sort-Object {
        if ($_.FullName -eq $metaPath) { "0-meta.json" }
        else { "1-" + $_.FullName }
    }
    foreach ($file in $files) {
        $entryName = if ($file.FullName -eq $metaPath) {
            "meta.json"
        } else {
            "Custom/Scripts/AgenticCreator/VaMender/VaMender.dll"
        }
        $entry = $archive.CreateEntry(
            $entryName,
            [IO.Compression.CompressionLevel]::Optimal
        )
        $entry.LastWriteTime = $timestamp
        $input = [IO.File]::OpenRead($file.FullName)
        $output = $entry.Open()
        try {
            $input.CopyTo($output)
        } finally {
            $output.Dispose()
            $input.Dispose()
        }
    }
} finally {
    $archive.Dispose()
}

$verify = [IO.Compression.ZipFile]::OpenRead($resolvedOutput)
try {
    $names = @($verify.Entries | ForEach-Object FullName)
    if ($names[0] -ne "meta.json") {
        throw "VAR metadata must be the first archive member"
    }
    if ($names | Where-Object {
        $_.StartsWith("/") -or $_.Contains("../") -or $_.Contains("\")
    }) {
        throw "VAR contains an unsafe or non-normalized member path"
    }
    foreach ($required in @(
        "meta.json",
        "Custom/Scripts/AgenticCreator/VaMender/VaMender.dll"
    )) {
        if ($names -notcontains $required) {
            throw "VAR is missing required member: $required"
        }
    }
    if ($names | Where-Object {
        $_.EndsWith(".cs") -or $_.EndsWith(".cslist")
    }) {
        throw "Release VAR must not expose dynamically compiled C# sources"
    }
} finally {
    $verify.Dispose()
}

$hash = (Get-FileHash -LiteralPath $resolvedOutput -Algorithm SHA256).Hash.ToLowerInvariant()
Write-Output "Created $resolvedOutput"
Write-Output "SHA-256 $hash"
Write-Output "Precompiled VaMender.dll SHA-256 $pluginAssemblyHash"
Write-Output "In-VaM VaMender control panel VAR: no executable embedded"

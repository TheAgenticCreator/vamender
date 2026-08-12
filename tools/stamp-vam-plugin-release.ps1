# SPDX-License-Identifier: MIT

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^0\.[0-9]\.[0-9]$')]
    [string]$Version,
    [string]$InputPath =
        ".\vam-plugin\Custom\Scripts\AgenticCreator\VaMender\VaMender.dll",
    [string]$OutputPath = ".\target\stamped-plugin\VaMender.dll"
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
$sourceRoot = Join-Path $projectRoot (
    "vam-plugin\Custom\Scripts\AgenticCreator\VaMender\src"
)
$baselineVersion = "0.1.0"
$baselineUrl =
    "https://github.com/TheAgenticCreator/vamender/releases/latest"
$releaseUrl =
    "https://github.com/TheAgenticCreator/vamender/releases?beta=1"
$baselineSetupText = "official VaMender Setup"
$releaseSetupText = "latest beta Setup build"
$baselineAssemblyHash =
    "ddb739c7f78a368914216f443f47877e0d7df619f2146c6fd64b482d02261d50"
$normalizedSourceHash =
    "768eb254b5a17da04d6b159a8680ca8daf75be1aaf9bc276e6fba8b612d42f11"

function Get-Sha256Bytes {
    param([Parameter(Mandatory = $true)][byte[]]$Bytes)

    $algorithm = [Security.Cryptography.SHA256]::Create()
    try {
        $digest = $algorithm.ComputeHash($Bytes)
    } finally {
        $algorithm.Dispose()
    }
    return ([BitConverter]::ToString($digest)).Replace(
        "-",
        ""
    ).ToLowerInvariant()
}

function Get-NormalizedSourceHash {
    $payload = ""
    $versionDeclaration = "private const string Version = `"$Version`";"
    $baselineDeclaration =
        "private const string Version = `"$baselineVersion`";"
    $files = Get-ChildItem -LiteralPath $sourceRoot -Filter "*.cs" -File |
        Sort-Object Name
    foreach ($file in $files) {
        $text = (Get-Content -LiteralPath $file.FullName -Raw).Replace(
            "`r`n",
            "`n"
        )
        $text = $text.Replace($versionDeclaration, $baselineDeclaration)
        $text = $text.Replace($releaseUrl, $baselineUrl)
        $text = $text.Replace($releaseSetupText, $baselineSetupText)
        $payload += $file.Name + "`n" + $text + "`n"
    }
    return Get-Sha256Bytes ([Text.Encoding]::UTF8.GetBytes($payload))
}

function Replace-ExactUnicodeString {
    param(
        [Parameter(Mandatory = $true)][byte[]]$Bytes,
        [Parameter(Mandatory = $true)][string]$OldValue,
        [Parameter(Mandatory = $true)][string]$NewValue,
        [Parameter(Mandatory = $true)][int]$ExpectedCount
    )

    $oldBytes = [Text.Encoding]::Unicode.GetBytes($OldValue)
    $newBytes = [Text.Encoding]::Unicode.GetBytes($NewValue)
    if ($oldBytes.Length -ne $newBytes.Length) {
        throw "Release stamp strings must have equal UTF-16 lengths"
    }

    $positions = [Collections.Generic.List[int]]::new()
    for ($offset = 0; $offset -le $Bytes.Length - $oldBytes.Length; $offset++) {
        $matches = $true
        for ($index = 0; $index -lt $oldBytes.Length; $index++) {
            if ($Bytes[$offset + $index] -ne $oldBytes[$index]) {
                $matches = $false
                break
            }
        }
        if ($matches) {
            $positions.Add($offset)
            $offset += $oldBytes.Length - 1
        }
    }
    if ($positions.Count -ne $ExpectedCount) {
        throw (
            "Expected $ExpectedCount embedded occurrence(s) of '$OldValue', " +
            "found $($positions.Count). A native VaM rebuild is required."
        )
    }
    foreach ($position in $positions) {
        [Array]::Copy($newBytes, 0, $Bytes, $position, $newBytes.Length)
    }
}

$resolvedInput = [IO.Path]::GetFullPath((Join-Path $projectRoot $InputPath))
$resolvedOutput = [IO.Path]::GetFullPath((Join-Path $projectRoot $OutputPath))
if ($resolvedInput -eq $resolvedOutput) {
    throw "InputPath and OutputPath must differ"
}
if (-not (Test-Path -LiteralPath $resolvedInput -PathType Leaf)) {
    throw "Validated plugin baseline is missing: $resolvedInput"
}

$actualSourceHash = Get-NormalizedSourceHash
if ($actualSourceHash -ne $normalizedSourceHash) {
    throw (
        "Plugin source contains changes beyond the approved release stamp. " +
        "Expected normalized SHA-256 $normalizedSourceHash, got " +
        "$actualSourceHash. Rebuild and validate the DLL against VaM 1.22.0.13."
    )
}

$bytes = [IO.File]::ReadAllBytes($resolvedInput)
$actualAssemblyHash = Get-Sha256Bytes $bytes
if ($actualAssemblyHash -ne $baselineAssemblyHash) {
    throw (
        "Plugin DLL is not the approved CLR 2 baseline. Expected SHA-256 " +
        "$baselineAssemblyHash, got $actualAssemblyHash."
    )
}

Replace-ExactUnicodeString $bytes $baselineVersion $Version 2
Replace-ExactUnicodeString $bytes $baselineUrl $releaseUrl 2
Replace-ExactUnicodeString $bytes $baselineSetupText $releaseSetupText 1

$outputDirectory = Split-Path -Parent $resolvedOutput
New-Item -ItemType Directory -Force -Path $outputDirectory | Out-Null
[IO.File]::WriteAllBytes($resolvedOutput, $bytes)
$outputHash = Get-Sha256Bytes $bytes

Write-Output "Stamped validated CLR 2 plugin: $resolvedOutput"
Write-Output "Version: $Version"
Write-Output "Normalized source SHA-256: $actualSourceHash"
Write-Output "Stamped DLL SHA-256: $outputHash"

# SPDX-License-Identifier: MIT

[CmdletBinding()]
param(
    [string]$EnginePath = ".\target\release\vamender.exe",
    [string]$OutputRoot = "",
    [switch]$KeepArtifacts
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
$scenarioMatrixPath = Join-Path $projectRoot "tests\synthetic-var-corpus\scenario-matrix.json"
$compressionGenerator = Join-Path $projectRoot (
    "tests\synthetic-var-corpus\build_compressed_var.py"
)
$python = Get-Command python.exe -ErrorAction SilentlyContinue

if (-not (Test-Path -LiteralPath $EnginePath -PathType Leaf)) {
    throw "VaMender engine was not found: $EnginePath"
}
if (-not (Test-Path -LiteralPath $scenarioMatrixPath -PathType Leaf)) {
    throw "Synthetic scenario matrix is missing: $scenarioMatrixPath"
}
if (-not (Test-Path -LiteralPath $compressionGenerator -PathType Leaf)) {
    throw "Compressed VAR fixture generator is missing: $compressionGenerator"
}
if ($null -eq $python) {
    throw "Python 3 is required to generate the BZIP2 and LZMA VAR fixtures"
}

$EnginePath = [IO.Path]::GetFullPath($EnginePath)
if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $OutputRoot = Join-Path $env:TEMP (
        "VaMender-Release-Scenarios-" + (Get-Date -Format "yyyyMMdd-HHmmss")
    )
}
$OutputRoot = [IO.Path]::GetFullPath($OutputRoot)
New-Item -ItemType Directory -Force -Path $OutputRoot | Out-Null

Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem

$scenarioMatrix = @(
    Get-Content -LiteralPath $scenarioMatrixPath -Raw | ConvertFrom-Json
)
$script:Results = [System.Collections.Generic.List[object]]::new()
$script:CurrentScenarioRoot = $null
$script:CurrentCommands = $null
$script:Screenshots = [System.Collections.Generic.List[string]]::new()

function Assert-True {
    param(
        [Parameter(Mandatory = $true)] [bool]$Condition,
        [Parameter(Mandatory = $true)] [string]$Message
    )

    if (-not $Condition) {
        throw $Message
    }
}

function Assert-File {
    param([Parameter(Mandatory = $true)] [string]$Path)

    Assert-True (Test-Path -LiteralPath $Path -PathType Leaf) "Expected file is missing: $Path"
}

function Assert-Directory {
    param([Parameter(Mandatory = $true)] [string]$Path)

    Assert-True (Test-Path -LiteralPath $Path -PathType Container) "Expected directory is missing: $Path"
}

function Assert-TextContains {
    param(
        [Parameter(Mandatory = $true)] [string]$Path,
        [Parameter(Mandatory = $true)] [string]$Expected
    )

    Assert-File $Path
    $text = Get-Content -LiteralPath $Path -Raw
    Assert-True $text.Contains($Expected) "Expected '$Expected' in $Path"
}

function Assert-TextNotContains {
    param(
        [Parameter(Mandatory = $true)] [string]$Path,
        [Parameter(Mandatory = $true)] [string]$Unexpected
    )

    Assert-File $Path
    $text = Get-Content -LiteralPath $Path -Raw
    Assert-True (-not $text.Contains($Unexpected)) "Did not expect '$Unexpected' in $Path"
}

function Write-VarArchive {
    param(
        [Parameter(Mandatory = $true)] [string]$Path,
        [Parameter(Mandatory = $true)] [hashtable]$Members
    )

    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $Path) | Out-Null
    Remove-Item -LiteralPath $Path -Force -ErrorAction SilentlyContinue
    $archive = [IO.Compression.ZipFile]::Open(
        $Path,
        [IO.Compression.ZipArchiveMode]::Create
    )
    try {
        foreach ($name in ($Members.Keys | Sort-Object)) {
            $entry = $archive.CreateEntry(
                $name,
                [IO.Compression.CompressionLevel]::Optimal
            )
            $stream = $entry.Open()
            try {
                $value = $Members[$name]
                $bytes = if ($value -is [byte[]]) {
                    $value
                } else {
                    [Text.Encoding]::UTF8.GetBytes([string]$value)
                }
                $stream.Write($bytes, 0, $bytes.Length)
            } finally {
                $stream.Dispose()
            }
        }
    } finally {
        $archive.Dispose()
    }
}

function Write-Meta {
    param(
        [Parameter(Mandatory = $true)] [string]$Id,
        [hashtable]$Dependencies = @{},
        [string]$License = "CC BY"
    )

    $parts = $Id.Split('.')
    if ($parts.Count -lt 3) {
        throw "Synthetic VAR ID is invalid: $Id"
    }
    return (@{
        name = $Id
        creatorName = $parts[0]
        packageName = $parts[1]
        licenseType = $License
        description = "Synthetic VaMender release fixture"
        dependencies = $Dependencies
    } | ConvertTo-Json -Depth 8 -Compress)
}

function Write-CompressedVar {
    param(
        [Parameter(Mandatory = $true)] [string]$Path,
        [Parameter(Mandatory = $true)] [ValidateSet("bzip2", "lzma")] [string]$Compression,
        [Parameter(Mandatory = $true)] [string]$Id
    )

    & $python.Source $compressionGenerator $Path $Compression $Id
    if ($LASTEXITCODE -ne 0) {
        throw "Could not create $Compression synthetic VAR fixture: $Path"
    }
}

function Get-PackageSnapshot {
    param([Parameter(Mandatory = $true)] [string]$Root)

    $rootPath = [IO.Path]::GetFullPath($Root).TrimEnd('\')
    return @(
        Get-ChildItem -LiteralPath $rootPath -File -Recurse |
            Sort-Object FullName |
            ForEach-Object {
                $relative = $_.FullName.Substring($rootPath.Length).TrimStart('\').Replace('\', '/')
                "$relative|$((Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant())"
            }
    ) -join "`n"
}

function Get-ArchiveEntryText {
    param(
        [Parameter(Mandatory = $true)] [string]$ArchivePath,
        [Parameter(Mandatory = $true)] [string]$EntryName
    )

    $archive = [IO.Compression.ZipFile]::OpenRead($ArchivePath)
    try {
        $entry = $archive.GetEntry($EntryName)
        if ($null -eq $entry) {
            throw "Archive entry is missing: $ArchivePath :: $EntryName"
        }
        $reader = [IO.StreamReader]::new($entry.Open(), [Text.Encoding]::UTF8, $true)
        try {
            return $reader.ReadToEnd()
        } finally {
            $reader.Dispose()
        }
    } finally {
        $archive.Dispose()
    }
}

function Get-ManifestRecords {
    param([Parameter(Mandatory = $true)] [string]$ManifestPath)

    Assert-File $ManifestPath
    return @(
        Get-Content -LiteralPath $ManifestPath |
            Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
            ForEach-Object { $_ | ConvertFrom-Json }
    )
}

function Assert-ReportSet {
    param([Parameter(Mandatory = $true)] [string]$ReportRoot)

    foreach ($name in @(
        "actions_taken.txt",
        "actions_required.txt",
        "missing_dependencies.txt"
    )) {
        Assert-File (Join-Path $ReportRoot $name)
    }
}

function Invoke-Engine {
    param(
        [Parameter(Mandatory = $true)] [string]$Label,
        [Parameter(Mandatory = $true)] [string[]]$Arguments,
        [bool]$ExpectedSuccess = $true
    )

    $outputPath = Join-Path $script:CurrentScenarioRoot "engine-$Label.txt"
    $lines = @(& $EnginePath @Arguments 2>&1)
    $exitCode = $LASTEXITCODE
    $lines | ForEach-Object { $_.ToString() } | Set-Content -LiteralPath $outputPath
    $record = [pscustomobject]@{
        name = $Label
        exit_code = $exitCode
        expected_success = $ExpectedSuccess
        output = $outputPath
    }
    $script:CurrentCommands.Add($record)
    if ($ExpectedSuccess -and $exitCode -ne 0) {
        throw "Engine command '$Label' failed with exit code $exitCode. See $outputPath"
    }
    if (-not $ExpectedSuccess -and $exitCode -eq 0) {
        throw "Engine command '$Label' unexpectedly succeeded. See $outputPath"
    }
    return $record
}

function New-ScenarioRoot {
    param([Parameter(Mandatory = $true)] [string]$Id)

    $root = Join-Path $OutputRoot $Id
    New-Item -ItemType Directory -Force -Path (Join-Path $root "AddonPackages") | Out-Null
    New-Item -ItemType Directory -Force -Path (Join-Path $root "reports") | Out-Null
    return $root
}

function Invoke-Scenario {
    param(
        [Parameter(Mandatory = $true)] [string]$Id,
        [Parameter(Mandatory = $true)] [scriptblock]$Action
    )

    $definition = @($scenarioMatrix | Where-Object { $_.id -eq $Id })
    if ($definition.Count -ne 1) {
        throw "Scenario matrix must contain exactly one entry for $Id"
    }
    $root = New-ScenarioRoot $Id
    $script:CurrentScenarioRoot = $root
    $script:CurrentCommands = [System.Collections.Generic.List[object]]::new()
    try {
        & $Action $root
        $script:Results.Add([pscustomobject]@{
            id = $Id
            title = $definition[0].title
            requirements = @($definition[0].requirements)
            passed = $true
            root = $root
            commands = @($script:CurrentCommands)
        })
    } catch {
        $script:Results.Add([pscustomobject]@{
            id = $Id
            title = $definition[0].title
            requirements = @($definition[0].requirements)
            passed = $false
            root = $root
            commands = @($script:CurrentCommands)
            failure = $_.Exception.Message
        })
        throw "Synthetic scenario $Id failed: $($_.Exception.Message)"
    }
}

function Save-TextScreenshot {
    param(
        [Parameter(Mandatory = $true)] [string]$Path,
        [Parameter(Mandatory = $true)] [string]$Title,
        [Parameter(Mandatory = $true)] [string[]]$Lines
    )

    Add-Type -AssemblyName System.Drawing
    $font = New-Object Drawing.Font("Consolas", 20)
    $titleFont = New-Object Drawing.Font("Segoe UI", 28, ([Drawing.FontStyle]::Bold))
    $width = 1600
    $lineHeight = 31
    $height = [Math]::Max(600, 118 + ($Lines.Count * $lineHeight))
    $bitmap = New-Object Drawing.Bitmap($width, $height)
    $graphics = [Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.Clear([Drawing.Color]::FromArgb(25, 29, 36))
        $graphics.TextRenderingHint = [Drawing.Text.TextRenderingHint]::ClearTypeGridFit
        $graphics.DrawString($Title, $titleFont, [Drawing.Brushes]::White, 42, 28)
        $graphics.DrawString(
            "VaMender synthetic release evidence",
            (New-Object Drawing.Font("Segoe UI", 15)),
            [Drawing.Brushes]::LightGray,
            45,
            72
        )
        $y = 118
        foreach ($line in $Lines) {
            $text = [string]$line
            if ($text.Length -gt 112) {
                $text = $text.Substring(0, 109) + "..."
            }
            $graphics.DrawString($text, $font, [Drawing.Brushes]::Gainsboro, 45, $y)
            $y += $lineHeight
        }
        $bitmap.Save($Path, [Drawing.Imaging.ImageFormat]::Png)
        $script:Screenshots.Add($Path)
    } finally {
        $graphics.Dispose()
        $bitmap.Dispose()
        $font.Dispose()
        $titleFont.Dispose()
    }
}

Invoke-Scenario "01-read-only-inventory" {
    param($root)
    $packages = Join-Path $root "AddonPackages"
    Write-VarArchive (Join-Path $packages "Demo.Provider.1.var") @{
        "meta.json" = Write-Meta "Demo.Provider.1"
        "Custom/Assets/Demo/provider.txt" = "provider payload"
    }
    Write-VarArchive (Join-Path $packages "nested\Nokisaki.时崎狂三.1.VAR") @{
        "meta.json" = Write-Meta "Nokisaki.时崎狂三.1"
        "Custom/Assets/Nokisaki/hair.txt" = "Unicode nested payload"
    }
    Write-VarArchive (Join-Path $packages "Demo.Consumer.1.var") @{
        "meta.json" = Write-Meta "Demo.Consumer.1" @{ "Demo.Provider.1" = @{} }
        "Saves/scene/consumer.json" = '{"url":"Demo.Provider.1:/Custom/Assets/Demo/provider.txt"}'
    }
    $before = Get-PackageSnapshot $packages
    $check = Join-Path $root "reports\check"
    $deep = Join-Path $root "reports\deep"
    Invoke-Engine -Label "check" -Arguments @("check", $packages, "--out", $check) | Out-Null
    Invoke-Engine -Label "deep-check" -Arguments @("check", $packages, "--deep", "--out", $deep) | Out-Null
    Assert-True ((Get-PackageSnapshot $packages) -eq $before) "Read-only inventory changed a synthetic VAR"
    Assert-ReportSet $check
    Assert-ReportSet $deep
    Assert-TextContains (Join-Path $check "actions_required.txt") "0 invalid archives"
    Save-TextScreenshot (Join-Path $OutputRoot "01-clean-inventory.png") "01  Read-only inventory" @(
        "check: PASS", "check --deep: PASS", "Nested and Unicode VARs retained", "No package bytes or paths changed"
    )
}

Invoke-Scenario "02-vam-log-planning" {
    param($root)
    $packages = Join-Path $root "AddonPackages"
    Write-VarArchive (Join-Path $packages "Demo.Broken.1.var") @{
        "meta.json" = Write-Meta "Demo.Broken.1" @{ "Demo.Missing.1" = @{} }
        "Saves/scene/broken.json" = '{"url":"Demo.Missing.1:/Custom/Assets/Demo/missing.txt"}'
    }
    $before = Get-PackageSnapshot $packages
    $freshLog = Join-Path $root "fresh-output_log.txt"
    Set-Content -LiteralPath $freshLog -Value "Missing addon package Demo.Missing.1 that package Demo.Broken.1 depends on"
    $freshReport = Join-Path $root "reports\fresh"
    Invoke-Engine -Label "fresh-plan" -Arguments @("plan", $packages, "--vam-log", $freshLog, "--out", $freshReport) | Out-Null
    Assert-ReportSet $freshReport
    Assert-TextContains (Join-Path $freshReport "actions_required.txt") "Using fresh VaM package log"
    Assert-TextContains (Join-Path $freshReport "missing_dependencies.txt") "Demo.Missing.1"

    $staleLog = Join-Path $root "stale-output_log.txt"
    Set-Content -LiteralPath $staleLog -Value "Missing addon package Demo.Missing.1 that package Demo.Broken.1 depends on"
    (Get-Item -LiteralPath $staleLog).LastWriteTime = (Get-Date).AddDays(-2)
    $staleReport = Join-Path $root "reports\stale"
    Invoke-Engine -Label "stale-plan" -Arguments @("plan", $packages, "--vam-log", $staleLog, "--out", $staleReport) | Out-Null
    Assert-TextContains (Join-Path $staleReport "actions_required.txt") "VaM log is older"
    Assert-TextNotContains (Join-Path $staleReport "actions_required.txt") "Using fresh VaM package log"

    $absentReport = Join-Path $root "reports\absent"
    Invoke-Engine -Label "absent-plan" -Arguments @("plan", $packages, "--out", $absentReport) | Out-Null
    Assert-TextNotContains (Join-Path $absentReport "actions_required.txt") "Using fresh VaM package log"
    Assert-True ((Get-PackageSnapshot $packages) -eq $before) "Planning changed a synthetic VAR"
    Save-TextScreenshot (Join-Path $OutputRoot "02-missing-dependency-plan.png") "02  VaM-log-aware planning" @(
        "Fresh log confirms Demo.Missing.1", "Stale and absent logs are not treated as runtime proof", "No mutation performed"
    )
}

Invoke-Scenario "03-metadata-repair" {
    param($root)
    $packages = Join-Path $root "AddonPackages"
    Write-VarArchive (Join-Path $packages "Demo.NoMeta.1.var") @{
        "Saves/scene/no-meta.json" = '{"dependency":"Demo.Real.1"}'
    }
    Write-VarArchive (Join-Path $packages "Demo.InvalidMeta.1.var") @{
        "meta.json" = "{invalid JSON"
        "Saves/scene/invalid-meta.json" = "{}"
    }
    Write-VarArchive (Join-Path $packages "Demo.FalseLabels.1.var") @{
        "meta.json" = '{"name":"Demo.FalseLabels.1","creatorName":"Demo","packageName":"FalseLabels","licenseType":"CC BY","dependencies":{"Demo.Real.1":{},"base.spec.1001":{},"ears_0.12_low_defaultMat_Normal.1001":{}}}'
        "Saves/scene/labels.json" = '{"url":"Demo.Real.1:/Custom/Assets/real.asset","material":"base.spec.1001","name":"ears_0.12_low_defaultMat_Normal.1001"}'
    }
    $beforeDryRun = Get-PackageSnapshot $packages
    $dryReport = Join-Path $root "reports\dry-run"
    Invoke-Engine -Label "repair-dry-run" -Arguments @("repair", $packages, "--out", $dryReport) | Out-Null
    Assert-True ((Get-PackageSnapshot $packages) -eq $beforeDryRun) "Dry-run metadata repair changed a synthetic VAR"
    $backup = Join-Path $root "backup"
    $repairReport = Join-Path $root "reports\repair"
    Invoke-Engine -Label "repair-apply" -Arguments @("repair", $packages, "--apply", "--license", "CC BY", "--non-interactive", "--backup", $backup, "--out", $repairReport) | Out-Null
    Assert-ReportSet $repairReport
    $noMeta = Get-ArchiveEntryText (Join-Path $packages "Demo.NoMeta.1.var") "meta.json" | ConvertFrom-Json
    $invalidMeta = Get-ArchiveEntryText (Join-Path $packages "Demo.InvalidMeta.1.var") "meta.json" | ConvertFrom-Json
    $labels = Get-ArchiveEntryText (Join-Path $packages "Demo.FalseLabels.1.var") "meta.json"
    Assert-True ($noMeta.licenseType -eq "CC BY") "Missing metadata was not rebuilt with the explicit license"
    Assert-True ($invalidMeta.licenseType -eq "CC BY") "Invalid metadata was not rebuilt with the explicit license"
    Assert-True $labels.Contains("Demo.Real.1") "Explicit VAR resource reference was lost during metadata repair"
    Assert-True (-not $labels.Contains("base.spec.1001")) "Material label was converted into a dependency"
    Assert-True (-not $labels.Contains("ears_0.12_low_defaultMat_Normal.1001")) "Descriptive label was converted into a dependency"
    $records = Get-ManifestRecords (Join-Path $backup "manifest.jsonl")
    Assert-True ($records.Count -ge 3) "Metadata mutations were not backed up and manifested"
    Save-TextScreenshot (Join-Path $OutputRoot "03-metadata-repair.png") "03  Conservative metadata repair" @(
        "Missing and invalid metadata rebuilt with explicit CC BY", "Real VAR references retained", "Material and descriptive labels excluded", "Whole-VAR manifest records verified"
    )
}

Invoke-Scenario "04-archive-compatibility" {
    param($root)
    $packages = Join-Path $root "AddonPackages"
    Write-CompressedVar (Join-Path $packages "Demo.Bzip.1.var") "bzip2" "Demo.Bzip.1"
    Write-CompressedVar (Join-Path $packages "Demo.Lzma.1.var") "lzma" "Demo.Lzma.1"
    Write-VarArchive (Join-Path $packages "Demo.CrcBroken.1.var") @{
        "meta.json" = Write-Meta "Demo.CrcBroken.1"
        "Custom/Assets/Demo/payload.txt" = "CRC validation fixture payload"
    }
    $crcPath = Join-Path $packages "Demo.CrcBroken.1.var"
    [byte[]]$crcBytes = [IO.File]::ReadAllBytes($crcPath)
    $header = [Text.Encoding]::ASCII.GetBytes("PK" + [char]1 + [char]2)
    $offset = -1
    for ($index = 0; $index -le $crcBytes.Length - $header.Length; $index++) {
        if ($crcBytes[$index] -eq $header[0] -and $crcBytes[$index + 1] -eq $header[1] -and $crcBytes[$index + 2] -eq $header[2] -and $crcBytes[$index + 3] -eq $header[3]) {
            $offset = $index
            break
        }
    }
    Assert-True ($offset -ge 0) "Could not locate ZIP central directory for CRC fixture"
    $crcBytes[$offset + 16] = $crcBytes[$offset + 16] -bxor 0x01
    [IO.File]::WriteAllBytes($crcPath, $crcBytes)
    [IO.File]::WriteAllBytes(
        (Join-Path $packages "Demo.Corrupt.1.var"),
        [Text.Encoding]::ASCII.GetBytes("not a ZIP archive")
    )
    $before = Get-PackageSnapshot $packages
    $report = Join-Path $root "reports\deep"
    Invoke-Engine -Label "deep-compatibility-check" -Arguments @("check", $packages, "--deep", "--out", $report) | Out-Null
    Assert-True ((Get-PackageSnapshot $packages) -eq $before) "Compatibility diagnosis changed a synthetic VAR"
    Assert-ReportSet $report
    Assert-TextContains (Join-Path $report "actions_required.txt") "Demo.Corrupt.1.var"
    Assert-TextContains (Join-Path $report "actions_required.txt") "Demo.CrcBroken.1.var"
    Assert-TextContains (Join-Path $report "actions_required.txt") "Demo.Lzma.1.var"
    Assert-TextNotContains (Join-Path $report "actions_required.txt") "Demo.Bzip.1.var :: invalid"
    Assert-TextContains (Join-Path $report "actions_required.txt") "Compression method not supported"
    Save-TextScreenshot (Join-Path $OutputRoot "04-corrupt-archive.png") "04  Archive compatibility and diagnosis" @(
        "BZIP2 VAR passes deep validation", "Unsupported LZMA, CRC, and non-ZIP failures are diagnosed", "No invalid archive is rewritten without a safe source"
    )
}

Invoke-Scenario "05-filename-repair" {
    param($root)
    $packages = Join-Path $root "AddonPackages"
    Write-VarArchive (Join-Path $packages "Demo.Rename.7 (1).var") @{
        "meta.json" = Write-Meta "Demo.Rename.7"
    }
    $canonical = Join-Path $packages "Demo.Duplicate.3.var"
    Write-VarArchive $canonical @{
        "meta.json" = Write-Meta "Demo.Duplicate.3"
        "Custom/Assets/Demo/item.txt" = "identical"
    }
    Copy-Item -LiteralPath $canonical -Destination (Join-Path $packages "Demo.Duplicate.3_3.var")
    Write-VarArchive (Join-Path $packages "Demo.Collision.3.var") @{
        "meta.json" = Write-Meta "Demo.Collision.3"
        "Custom/Assets/Demo/item.txt" = "canonical"
    }
    Write-VarArchive (Join-Path $packages "Demo.Collision.3_3.var") @{
        "meta.json" = Write-Meta "Demo.Collision.3"
        "Custom/Assets/Demo/item.txt" = "different"
    }
    $backup = Join-Path $root "backup"
    $report = Join-Path $root "reports\repair"
    Invoke-Engine -Label "filename-repair" -Arguments @("repair", $packages, "--apply", "--license", "CC BY", "--non-interactive", "--backup", $backup, "--out", $report) | Out-Null
    Assert-True (Test-Path -LiteralPath (Join-Path $packages "Demo.Rename.7.var")) "Unambiguous filename was not repaired"
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $packages "Demo.Rename.7 (1).var"))) "Malformed filename remained after repair"
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $packages "Demo.Duplicate.3_3.var"))) "Byte-identical duplicate was not archived"
    Assert-True (Test-Path -LiteralPath (Join-Path $packages "Demo.Collision.3_3.var")) "Non-identical collision was modified"
    Assert-TextContains (Join-Path $report "actions_required.txt") "SKIP FILENAME COLLISION"
    Assert-True ((Get-ManifestRecords (Join-Path $backup "manifest.jsonl")).Count -ge 2) "Filename mutations lack verified backup records"
}

Invoke-Scenario "06-version-migration" {
    param($root)
    $packages = Join-Path $root "AddonPackages"
    foreach ($version in @(1, 2)) {
        Write-VarArchive (Join-Path $packages "Demo.Asset.$version.var") @{
            "meta.json" = Write-Meta "Demo.Asset.$version"
            "Custom/Assets/Demo/item.txt" = "identical resource"
        }
        Write-VarArchive (Join-Path $packages "Demo.Script.$version.var") @{
            "meta.json" = Write-Meta "Demo.Script.$version"
            "Custom/Scripts/Demo/script.cs" = "public class SyntheticScript {}"
        }
        Write-VarArchive (Join-Path $packages "Demo.Different.$version.var") @{
            "meta.json" = Write-Meta "Demo.Different.$version"
            "Custom/Assets/Demo/item.txt" = "different resource $version"
        }
    }
    foreach ($version in @(1, 2, 3)) {
        Write-VarArchive (Join-Path $packages "Demo.Conflict.$version.var") @{
            "meta.json" = Write-Meta "Demo.Conflict.$version"
            "Custom/Assets/Demo/conflict.txt" = "identical conflict resource"
        }
    }
    Write-VarArchive (Join-Path $packages "Demo.Scene.1.var") @{
        "meta.json" = Write-Meta "Demo.Scene.1" @{ "Demo.Asset.1" = @{} }
        "Saves/scene/scene.json" = '{"asset":"Demo.Asset.1","url":"Demo.Asset.1:/Custom/Assets/Demo/item.txt"}'
    }
    Write-VarArchive (Join-Path $packages "Demo.ConflictConsumer.1.var") @{
        "meta.json" = '{"name":"Demo.ConflictConsumer.1","creatorName":"Demo","packageName":"ConflictConsumer","licenseType":"CC BY","dependencies":{"Demo.Conflict.1":{"preset":"one"},"Demo.Conflict.2":{"preset":"two"}}}'
        "Saves/scene/conflict.json" = "{}"
    }
    $backup = Join-Path $root "backup"
    $report = Join-Path $root "reports\migration"
    Invoke-Engine -Label "migrate" -Arguments @("migrate", $packages, "--apply", "--backup", $backup, "--out", $report) | Out-Null
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $packages "Demo.Asset.1.var"))) "Safe old version was not archived"
    Assert-True (Get-ArchiveEntryText (Join-Path $packages "Demo.Scene.1.var") "Saves/scene/scene.json").Contains("Demo.Asset.2") "Consumer reference was not migrated"
    Assert-True (Test-Path -LiteralPath (Join-Path $packages "Demo.Script.1.var")) "Script package version was incorrectly retired"
    Assert-True (Test-Path -LiteralPath (Join-Path $packages "Demo.Different.1.var")) "Non-identical provider version was incorrectly retired"
    Assert-True (Test-Path -LiteralPath (Join-Path $packages "Demo.Conflict.1.var")) "Metadata-conflicting provider was incorrectly retired"
    Assert-True (Test-Path -LiteralPath (Join-Path $packages "Demo.Conflict.2.var")) "Second metadata-conflicting provider was incorrectly retired"
    Assert-ReportSet $report
    Save-TextScreenshot (Join-Path $OutputRoot "05-migration-restore.png") "05  Version migration safety" @(
        "Byte-identical non-plugin version migrated", "Script and non-identical packages retained", "Metadata-payload conflict blocked retirement", "Backup manifest recorded every mutation"
    )
}

Invoke-Scenario "07-dependency-closure" {
    param($root)
    $packages = Join-Path $root "AddonPackages"
    Write-VarArchive (Join-Path $packages "Demo.Provider.2.var") @{
        "meta.json" = Write-Meta "Demo.Provider.2"
        "Custom/Assets/Demo/item.txt" = "new provider"
    }
    Write-VarArchive (Join-Path $packages "Demo.Relink.1.var") @{
        "meta.json" = Write-Meta "Demo.Relink.1" @{ "Demo.Provider.1" = @{} }
        "Saves/scene/relink.json" = '{"url":"Demo.Provider.1:/Custom/Assets/Demo/item.txt"}'
    }
    Write-VarArchive (Join-Path $packages "Demo.Broken.1.var") @{
        "meta.json" = Write-Meta "Demo.Broken.1" @{ "Demo.Missing.1" = @{} }
        "Saves/scene/broken.json" = '{"url":"Demo.Missing.1:/Custom/Assets/Demo/missing.txt"}'
    }
    Write-VarArchive (Join-Path $packages "Demo.Clothes.3.var") @{
        "meta.json" = Write-Meta "Demo.Clothes.3"
        "Custom/Clothing/Demo/Present.vam" = "{}"
    }
    Write-VarArchive (Join-Path $packages "Demo.MemberOwner.1.var") @{
        "meta.json" = Write-Meta "Demo.MemberOwner.1" @{ "Demo.Clothes.latest" = @{} }
        "Saves/scene/member.json" = '{"url":"Demo.Clothes.latest:/Custom/Clothing/Demo/Missing.vam"}'
    }
    Write-VarArchive (Join-Path $packages "Demo.Safe.1.var") @{
        "meta.json" = Write-Meta "Demo.Safe.1"
        "Custom/Assets/Demo/safe.txt" = "safe"
    }
    $log = Join-Path $root "output_log.txt"
    Set-Content -LiteralPath $log -Value "!> Clothing item Demo.Clothes.3:/Custom/Clothing/Demo/Missing.vam is missing"
    $plan = Join-Path $root "reports\plan"
    Invoke-Engine -Label "member-plan" -Arguments @("plan", $packages, "--vam-log", $log, "--out", $plan) | Out-Null
    Assert-TextContains (Join-Path $plan "actions_required.txt") "VaM-confirmed missing internal members in the fresh log: 1"
    $backup = Join-Path $root "backup"
    $run = Join-Path $root "reports\run"
    Invoke-Engine -Label "dependency-run" -Arguments @("run", $packages, "--backup", $backup, "--license", "CC BY", "--out", $run) | Out-Null
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $packages "Demo.Broken.1.var"))) "Broken dependency closure was not quarantined"
    Assert-True (Get-ArchiveEntryText (Join-Path $packages "Demo.Relink.1.var") "Saves/scene/relink.json").Contains("Demo.Provider.2") "Safe non-plugin relink did not occur"
    Assert-True (Test-Path -LiteralPath (Join-Path $packages "Demo.Safe.1.var")) "Unrelated safe package was modified"
    Assert-True (Test-Path -LiteralPath (Join-Path $packages "Demo.MemberOwner.1.var")) "Missing-member diagnosis incorrectly quarantined the owner"
    Assert-Directory (Join-Path $run "_details")
    Save-TextScreenshot (Join-Path $OutputRoot "06-broken-library-run.png") "06  Dependency closure isolation" @(
        "Broken package quarantined only after verified backup", "Safe non-plugin reference relinked locally", "Missing internal member remains a diagnostic, not invented content", "Unrelated VAR retained"
    )
}

Invoke-Scenario "08-mutation-and-restore-safety" {
    param($root)
    $packages = Join-Path $root "AddonPackages"
    $oldPath = Join-Path $packages "Demo.RestoreAsset.1.var"
    Write-VarArchive $oldPath @{
        "meta.json" = Write-Meta "Demo.RestoreAsset.1"
        "Custom/Assets/Demo/item.txt" = "restore original"
    }
    $originalHash = (Get-FileHash -LiteralPath $oldPath -Algorithm SHA256).Hash.ToLowerInvariant()
    Write-VarArchive (Join-Path $packages "Demo.RestoreAsset.2.var") @{
        "meta.json" = Write-Meta "Demo.RestoreAsset.2"
        "Custom/Assets/Demo/item.txt" = "restore original"
    }
    Write-VarArchive (Join-Path $packages "Demo.RestoreScene.1.var") @{
        "meta.json" = Write-Meta "Demo.RestoreScene.1" @{ "Demo.RestoreAsset.1" = @{} }
        "Saves/scene/restore.json" = '{"url":"Demo.RestoreAsset.1:/Custom/Assets/Demo/item.txt"}'
    }
    $beforeRejected = Get-PackageSnapshot $packages
    Invoke-Engine -Label "repair-without-backup" -Arguments @("repair", $packages, "--apply") -ExpectedSuccess $false | Out-Null
    Invoke-Engine -Label "migrate-without-backup" -Arguments @("migrate", $packages, "--apply") -ExpectedSuccess $false | Out-Null
    Assert-True ((Get-PackageSnapshot $packages) -eq $beforeRejected) "Rejected mutation gate changed a synthetic VAR"
    $backup = Join-Path $root "backup"
    $migration = Join-Path $root "reports\migration"
    Invoke-Engine -Label "migrate-for-restore" -Arguments @("migrate", $packages, "--apply", "--backup", $backup, "--out", $migration) | Out-Null
    $manifest = Join-Path $backup "manifest.jsonl"
    $records = Get-ManifestRecords $manifest
    Assert-True ($records.Count -gt 0) "Migration did not create a restore manifest"
    Write-VarArchive $oldPath @{
        "meta.json" = Write-Meta "Demo.RestoreAsset.1"
        "Custom/Assets/Demo/item.txt" = "conflicting current content"
    }
    $conflictHash = (Get-FileHash -LiteralPath $oldPath -Algorithm SHA256).Hash.ToLowerInvariant()
    Invoke-Engine -Label "restore-skip-existing" -Arguments @("restore", $packages, $manifest) | Out-Null
    Assert-True ((Get-FileHash -LiteralPath $oldPath -Algorithm SHA256).Hash.ToLowerInvariant() -eq $conflictHash) "Restore overwrote an existing VAR without --overwrite"
    Invoke-Engine -Label "restore-overwrite" -Arguments @("restore", $packages, $manifest, "--overwrite") | Out-Null
    Assert-True ((Get-FileHash -LiteralPath $oldPath -Algorithm SHA256).Hash.ToLowerInvariant() -eq $originalHash) "Overwrite restore did not recover the checksum-verified original"
    Assert-Directory (Join-Path $backup "restore-conflicts")

    $tamperRoot = Join-Path $root "tamper"
    $tamperPackages = Join-Path $tamperRoot "AddonPackages"
    New-Item -ItemType Directory -Force -Path $tamperPackages | Out-Null
    $badBackup = Join-Path $tamperRoot "bad-backup.var"
    [IO.File]::WriteAllBytes($badBackup, [Text.Encoding]::ASCII.GetBytes("tampered backup"))
    $checksumManifest = Join-Path $tamperRoot "checksum-manifest.jsonl"
    (@{ operation = "test"; source = "synthetic"; relative_path = "Demo.Target.1.var"; backup = $badBackup; sha256 = "00" } | ConvertTo-Json -Compress) | Set-Content -LiteralPath $checksumManifest
    Invoke-Engine -Label "restore-checksum-reject" -Arguments @("restore", $tamperPackages, $checksumManifest) | Out-Null
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $tamperPackages "Demo.Target.1.var"))) "Restore accepted a bad backup checksum"
    $traversalManifest = Join-Path $tamperRoot "traversal-manifest.jsonl"
    (@{ operation = "test"; source = "synthetic"; relative_path = "..\outside.var"; backup = $badBackup; sha256 = "00" } | ConvertTo-Json -Compress) | Set-Content -LiteralPath $traversalManifest
    Invoke-Engine -Label "restore-traversal-reject" -Arguments @("restore", $tamperPackages, $traversalManifest) | Out-Null
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $root "outside.var"))) "Restore wrote outside the selected AddonPackages root"
}

Invoke-Scenario "09-bridge-protocol" {
    param($root)
    $packages = Join-Path $root "AddonPackages"
    Write-VarArchive (Join-Path $packages "Demo.Bridge.1.var") @{
        "meta.json" = Write-Meta "Demo.Bridge.1"
    }
    $backup = Join-Path $root "backup"
    $state = Join-Path $root "bridge-state"
    New-Item -ItemType Directory -Force -Path $state | Out-Null
    '{"id":"..\\outside","operation":"check","deep":false}' | Set-Content -LiteralPath (Join-Path $state "request.json")
    $invalidId = Invoke-Engine -Label "bridge-invalid-id" -Arguments @("bridge", $packages, "--backup", $backup, "--state", $state, "--once")
    Assert-TextContains $invalidId.output "FAILED: invalid request"
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $root "outside"))) "Bridge request ID escaped its state directory"

    '{"id":"1002","operation":"unknown","deep":false}' | Set-Content -LiteralPath (Join-Path $state "request.json")
    Invoke-Engine -Label "bridge-unknown-operation" -Arguments @("bridge", $packages, "--backup", $backup, "--state", $state, "--once") | Out-Null
    $unknownResponse = Get-Content -LiteralPath (Join-Path $state "response.json") -Raw | ConvertFrom-Json
    Assert-True (-not $unknownResponse.success) "Bridge allowlist accepted an unknown operation"

    '{"id":"1003","operation":"check","deep":true}' | Set-Content -LiteralPath (Join-Path $state "request.json")
    Invoke-Engine -Label "bridge-check" -Arguments @("bridge", $packages, "--backup", $backup, "--state", $state, "--once") | Out-Null
    $checkResponse = Get-Content -LiteralPath (Join-Path $state "response.json") -Raw | ConvertFrom-Json
    Assert-True $checkResponse.success "Bridge check request did not complete"
    Assert-ReportSet (Join-Path $state "reports\1003\check")
    Invoke-Engine -Label "bridge-backup-containment" -Arguments @("bridge", $packages, "--backup", (Join-Path $packages "unsafe-backup"), "--once") -ExpectedSuccess $false | Out-Null
}

Invoke-Scenario "10-support-report" {
    param($root)
    $packages = Join-Path $root "AddonPackages"
    Write-VarArchive (Join-Path $packages "Demo.SupportBroken.1.var") @{
        "meta.json" = Write-Meta "Demo.SupportBroken.1" @{ "Demo.Missing.1" = @{} }
        "Saves/scene/support.json" = '{"url":"Demo.Missing.1:/Custom/Assets/Demo/missing.txt"}'
    }
    $support = Join-Path $root "reports\support"
    Invoke-Engine -Label "support-report" -Arguments @("support-report", $packages, "--deep", "--out", $support) | Out-Null
    $bundle = @(Get-ChildItem -LiteralPath $support -Filter "*.zip" -File)
    Assert-True ($bundle.Count -eq 1) "Support report did not produce exactly one ZIP bundle"
    $archive = [IO.Compression.ZipFile]::OpenRead($bundle[0].FullName)
    try {
        $entryNames = @($archive.Entries | ForEach-Object FullName)
        Assert-True ($entryNames -contains "README_FIRST.txt") "Support bundle lacks review instructions"
        Assert-True (-not ($entryNames -contains "output_log.txt")) "Support bundle includes raw VaM log data"
        foreach ($entry in $archive.Entries) {
            $reader = [IO.StreamReader]::new($entry.Open(), [Text.Encoding]::UTF8, $true)
            try {
                $text = $reader.ReadToEnd()
                Assert-True (-not $text.Contains($OutputRoot)) "Support bundle leaked an absolute disposable path"
            } finally {
                $reader.Dispose()
            }
        }
    } finally {
        $archive.Dispose()
    }
    Save-TextScreenshot (Join-Path $OutputRoot "07-support-report.png") "07  Privacy-safe support report" @(
        "Local diagnostic ZIP created", "Raw VaM log and absolute paths excluded", "Full inventory remains opt-in", "Manual review is required before sharing"
    )
}

$expectedIds = @($scenarioMatrix | ForEach-Object id | Sort-Object)
$actualIds = @($script:Results | ForEach-Object id | Sort-Object)
Assert-True (($expectedIds -join ",") -eq ($actualIds -join ",")) "Scenario runner did not execute every matrix entry"
Assert-True (-not (@($script:Results | Where-Object { -not $_.passed }))) "At least one scenario failed"

$summary = [pscustomobject]@{
    engine = $EnginePath
    output_root = $OutputRoot
    generated_at_utc = [DateTime]::UtcNow.ToString("o")
    scenario_matrix = $scenarioMatrixPath
    results = @($script:Results)
    screenshots = @($script:Screenshots)
}
$summaryPath = Join-Path $OutputRoot "summary.json"
$summary | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $summaryPath
$summary | ConvertTo-Json -Depth 10
Write-Output "Scenario evidence: $OutputRoot"

if (-not $KeepArtifacts) {
    Write-Output "Synthetic fixtures are retained for review; remove the output root when no longer needed."
}

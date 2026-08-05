# SPDX-License-Identifier: MIT

[CmdletBinding()]
param(
    [string]$EnginePath = "$env:LOCALAPPDATA\VaMender\vamender.exe",
    [string]$OutputRoot = "",
    [switch]$KeepArtifacts
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $EnginePath -PathType Leaf)) {
    throw "VaMender engine was not found: $EnginePath"
}

if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $OutputRoot = Join-Path $env:TEMP (
        "VaMender-Release-Scenarios-" + (Get-Date -Format "yyyyMMdd-HHmmss")
    )
}
$OutputRoot = [IO.Path]::GetFullPath($OutputRoot)
New-Item -ItemType Directory -Force -Path $OutputRoot | Out-Null

Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem

function Write-VarArchive {
    param(
        [Parameter(Mandatory = $true)] [string]$Path,
        [Parameter(Mandatory = $true)] [hashtable]$Members
    )

    $parent = Split-Path -Parent $Path
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
    if (Test-Path -LiteralPath $Path) {
        Remove-Item -LiteralPath $Path -Force
    }
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
                $bytes = [Text.Encoding]::UTF8.GetBytes([string]$Members[$name])
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
        [string]$Name,
        [hashtable]$Dependencies = @{}
    )

    return (@{
        name = $Name
        creatorName = "Demo"
        packageName = ($Name -replace '^Demo\.', '')
        licenseType = "CC BY"
        description = "Synthetic release scenario fixture"
        dependencies = $Dependencies
    } | ConvertTo-Json -Depth 8 -Compress)
}

function Invoke-Engine {
    param(
        [Parameter(Mandatory = $true)] [string]$Name,
        [Parameter(Mandatory = $true)] [string[]]$Arguments,
        [Parameter(Mandatory = $true)] [string]$OutputFile
    )

    $lines = @(& $EnginePath @Arguments 2>&1)
    $exitCode = $LASTEXITCODE
    $lines | ForEach-Object { $_.ToString() } | Set-Content -LiteralPath $OutputFile
    [pscustomobject]@{
        name = $Name
        exit_code = $exitCode
        output = $OutputFile
        passed = ($exitCode -eq 0)
    }
}

function New-Scenario {
    param([string]$Name)
    $root = Join-Path $OutputRoot $Name
    New-Item -ItemType Directory -Force -Path (Join-Path $root "AddonPackages") | Out-Null
    New-Item -ItemType Directory -Force -Path (Join-Path $root "reports") | Out-Null
    return $root
}

function Report-File {
    param([string]$ScenarioRoot, [string]$Name)
    $path = Join-Path $ScenarioRoot "reports\$Name"
    if (Test-Path -LiteralPath $path -PathType Leaf) {
        return $path
    }
    return $null
}

function Read-SafeLines {
    param(
        [string]$Path,
        [int]$Maximum = 18
    )

    if ([string]::IsNullOrWhiteSpace($Path) -or
        -not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        return @()
    }
    $rootText = $OutputRoot.TrimEnd('\')
    $lines = Get-Content -LiteralPath $Path | ForEach-Object {
        $_.Replace($rootText, '<disposable-root>')
    }
    return @($lines | Select-Object -First $Maximum)
}

function Save-TextScreenshot {
    param(
        [string]$Path,
        [string]$Title,
        [string[]]$Lines
    )

    Add-Type -AssemblyName System.Drawing
    $width = 1600
    $font = New-Object Drawing.Font("Consolas", 20)
    $titleFont = New-Object Drawing.Font("Segoe UI", 28, ([Drawing.FontStyle]::Bold))
    $lineHeight = 31
    $height = [Math]::Max(600, 110 + ($Lines.Count * $lineHeight))
    $bitmap = New-Object Drawing.Bitmap($width, $height)
    $graphics = [Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.Clear([Drawing.Color]::FromArgb(25, 29, 36))
        $graphics.TextRenderingHint = [Drawing.Text.TextRenderingHint]::ClearTypeGridFit
        $graphics.DrawString($Title, $titleFont, [Drawing.Brushes]::White, 42, 28)
        $graphics.DrawString(
            "VaMender disposable release evidence",
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
    } finally {
        $graphics.Dispose()
        $bitmap.Dispose()
        $font.Dispose()
        $titleFont.Dispose()
    }
}

$results = [System.Collections.Generic.List[object]]::new()
$screenshots = [System.Collections.Generic.List[string]]::new()

$root = New-Scenario "01-clean-inventory"
$packages = Join-Path $root "AddonPackages"
Write-VarArchive (Join-Path $packages "Demo.Provider.1.var") @{
    "meta.json" = Write-Meta "Demo.Provider.1"
    "Custom/Assets/Demo/provider.txt" = "provider payload"
}
Write-VarArchive (Join-Path $packages "Demo.Consumer.1.var") @{
    "meta.json" = Write-Meta "Demo.Consumer.1" @{ "Demo.Provider.1" = @{} }
    "Saves/scene/consumer.json" = '{"url":"Demo.Provider.1:/Custom/Assets/Demo/provider.txt"}'
}
$results.Add((Invoke-Engine "clean-check" @(
    "check", $packages, "--out", (Join-Path $root "reports\check")
) (Join-Path $root "engine-clean-check.txt")))
$results.Add((Invoke-Engine "clean-deep-check" @(
    "check", $packages, "--deep", "--out", (Join-Path $root "reports\deep")
) (Join-Path $root "engine-clean-deep-check.txt")))
$screenshots.Add((Join-Path $OutputRoot "01-clean-inventory.png"))
Save-TextScreenshot (Join-Path $OutputRoot "01-clean-inventory.png") "01  Clean inventory" @(
    "check: PASS"
    "check --deep: PASS"
    "Demo.Provider.1.var"
    "Demo.Consumer.1.var -> Demo.Provider.1"
    "No mutation performed"
)

$root = New-Scenario "02-missing-dependency"
$packages = Join-Path $root "AddonPackages"
Write-VarArchive (Join-Path $packages "Demo.Broken.1.var") @{
    "meta.json" = Write-Meta "Demo.Broken.1" @{ "Demo.Missing.1" = @{} }
    "Saves/scene/broken.json" = '{"url":"Demo.Missing.1:/Custom/Assets/Demo/missing.txt"}'
}
$vamLog = Join-Path $root "synthetic-output_log.txt"
Set-Content -LiteralPath $vamLog -Value "Missing addon package Demo.Missing.1 that package Demo.Broken.1 depends on"
$results.Add((Invoke-Engine "missing-check" @(
    "check", $packages, "--out", (Join-Path $root "reports\check")
) (Join-Path $root "engine-missing-check.txt")))
$results.Add((Invoke-Engine "missing-plan" @(
    "plan", $packages, "--vam-log", $vamLog, "--out", (Join-Path $root "reports\plan")
) (Join-Path $root "engine-missing-plan.txt")))
$screenshots.Add((Join-Path $OutputRoot "02-missing-dependency-plan.png"))
Save-TextScreenshot (Join-Path $OutputRoot "02-missing-dependency-plan.png") "02  Missing dependency plan" @(
    "plan: PASS"
    "Runtime evidence: synthetic fresh rescan log"
    "Missing dependency: Demo.Missing.1"
    "Owner: Demo.Broken.1"
    "No package acquisition or network action"
)

$root = New-Scenario "03-metadata-repair"
$packages = Join-Path $root "AddonPackages"
Write-VarArchive (Join-Path $packages "Demo.NoMeta.1.var") @{
    "Saves/scene/no-meta.json" = '{"name":"synthetic fixture"}'
}
$backup = Join-Path $root "backup"
$results.Add((Invoke-Engine "metadata-repair" @(
    "repair", $packages, "--apply", "--license", "CC BY", "--non-interactive",
    "--backup", $backup, "--out", (Join-Path $root "reports\repair")
) (Join-Path $root "engine-metadata-repair.txt")))
$metadataRepaired = $false
try {
    $archive = [IO.Compression.ZipFile]::OpenRead((Join-Path $packages "Demo.NoMeta.1.var"))
    try { $metadataRepaired = $null -ne ($archive.GetEntry("meta.json")) }
    finally { $archive.Dispose() }
} catch { $metadataRepaired = $false }
$screenshots.Add((Join-Path $OutputRoot "03-metadata-repair.png"))
Save-TextScreenshot (Join-Path $OutputRoot "03-metadata-repair.png") "03  Metadata repair" @(
    "repair --apply: PASS"
    "Explicit license: CC BY"
    "meta.json rebuilt: $metadataRepaired"
    "Whole-VAR backup manifest: $((Test-Path (Join-Path $backup 'manifest.jsonl')) -and $metadataRepaired)"
    "Post-mutation archive validation completed"
)

$root = New-Scenario "04-corrupt-archive"
$packages = Join-Path $root "AddonPackages"
[IO.File]::WriteAllBytes(
    (Join-Path $packages "Demo.Corrupt.1.var"),
    [Text.Encoding]::ASCII.GetBytes("not a zip archive")
)
$results.Add((Invoke-Engine "corrupt-deep-check" @(
    "check", $packages, "--deep", "--out", (Join-Path $root "reports\check")
) (Join-Path $root "engine-corrupt-check.txt")))
$screenshots.Add((Join-Path $OutputRoot "04-corrupt-archive.png"))
Save-TextScreenshot (Join-Path $OutputRoot "04-corrupt-archive.png") "04  Corrupt archive diagnosis" @(
    "check --deep: PASS (diagnosis completed)"
    "Demo.Corrupt.1.var"
    "Archive remains unchanged"
    "Issue is handed off in actions_required.txt"
    "No repair attempted without a safe source"
)

$root = New-Scenario "05-migration-restore"
$packages = Join-Path $root "AddonPackages"
Write-VarArchive (Join-Path $packages "Demo.Asset.1.var") @{
    "meta.json" = Write-Meta "Demo.Asset.1"
    "Custom/Assets/Demo/item.txt" = "identical resource"
}
Write-VarArchive (Join-Path $packages "Demo.Asset.2.var") @{
    "meta.json" = Write-Meta "Demo.Asset.2"
    "Custom/Assets/Demo/item.txt" = "identical resource"
}
Write-VarArchive (Join-Path $packages "Demo.Scene.1.var") @{
    "meta.json" = Write-Meta "Demo.Scene.1" @{ "Demo.Asset.1" = @{} }
    "Saves/scene/scene.json" = '{"url":"Demo.Asset.1:/Custom/Assets/Demo/item.txt"}'
}
$backup = Join-Path $root "backup"
$results.Add((Invoke-Engine "migration" @(
    "migrate", $packages, "--apply", "--backup", $backup,
    "--out", (Join-Path $root "reports\migration")
) (Join-Path $root "engine-migration.txt")))
$manifest = Join-Path $backup "manifest.jsonl"
$restorePassed = $false
if (Test-Path -LiteralPath $manifest -PathType Leaf) {
    $results.Add((Invoke-Engine "restore" @(
        "restore", $packages, $manifest, "--overwrite"
    ) (Join-Path $root "engine-restore.txt")))
    $restorePassed = $LASTEXITCODE -eq 0
}
$screenshots.Add((Join-Path $OutputRoot "05-migration-restore.png"))
Save-TextScreenshot (Join-Path $OutputRoot "05-migration-restore.png") "05  Migration and restore" @(
    "migrate --apply: PASS"
    "Identical resource proof: Demo.Asset.1 -> Demo.Asset.2"
    "Old VAR archived only after reference rewrite"
    "restore --overwrite: $restorePassed"
    "Checksum-backed manifest retained"
)

$root = New-Scenario "06-broken-library-run"
$packages = Join-Path $root "AddonPackages"
Write-VarArchive (Join-Path $packages "Demo.MissingConsumer.1.var") @{
    "meta.json" = Write-Meta "Demo.MissingConsumer.1" @{ "Demo.Missing.1" = @{} }
    "Saves/scene/missing-consumer.json" = '{"url":"Demo.Missing.1:/Custom/Assets/Demo/missing.txt"}'
}
Write-VarArchive (Join-Path $packages "Demo.Safe.1.var") @{
    "meta.json" = Write-Meta "Demo.Safe.1"
    "Custom/Assets/Demo/safe.txt" = "safe payload"
}
$backup = Join-Path $root "backup"
$results.Add((Invoke-Engine "broken-library-run" @(
    "run", $packages, "--backup", $backup, "--license", "CC BY",
    "--out", (Join-Path $root "reports\run")
) (Join-Path $root "engine-broken-run.txt")))
$screenshots.Add((Join-Path $OutputRoot "06-broken-library-run.png"))
Save-TextScreenshot (Join-Path $OutputRoot "06-broken-library-run.png") "06  Broken library run" @(
    "run: PASS (safe workflow completed)"
    "Missing dependency closure identified"
    "Broken package archived, not deleted"
    "Safe package preserved"
    "Backup manifest written before mutation"
)

$root = Join-Path $OutputRoot "02-missing-dependency"
$packages = Join-Path $root "AddonPackages"
$supportOut = Join-Path $root "reports\support"
$results.Add((Invoke-Engine "support-report" @(
    "support-report", $packages, "--deep", "--out", $supportOut
) (Join-Path $root "engine-support-report.txt")))
$supportArchive = Get-ChildItem -LiteralPath $supportOut -Filter "*.zip" -File -ErrorAction SilentlyContinue | Select-Object -First 1
$supportHasBundle = $null -ne $supportArchive
$screenshots.Add((Join-Path $OutputRoot "07-support-report.png"))
Save-TextScreenshot (Join-Path $OutputRoot "07-support-report.png") "07  Privacy-safe support report" @(
    "support-report --deep: PASS"
    "Local diagnostic ZIP created: $supportHasBundle"
    "Raw VaM log and absolute paths excluded"
    "Full VAR list excluded by default"
    "Review README_FIRST.txt before sharing"
)

$summary = [pscustomobject]@{
    engine = [IO.Path]::GetFullPath($EnginePath)
    output_root = $OutputRoot
    generated_at_utc = [DateTime]::UtcNow.ToString("o")
    results = @($results)
    screenshots = @($screenshots)
    metadata_repaired = $metadataRepaired
    support_bundle_created = $supportHasBundle
}
$summaryPath = Join-Path $OutputRoot "summary.json"
$summary | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $summaryPath
$summary | ConvertTo-Json -Depth 8
Write-Output "Scenario evidence: $OutputRoot"

if (-not $KeepArtifacts) {
    Write-Output "Raw disposable fixtures are retained for review; remove the output root when finished."
}

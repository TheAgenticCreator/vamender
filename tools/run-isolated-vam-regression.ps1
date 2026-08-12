# SPDX-License-Identifier: MIT

[CmdletBinding()]
param(
    [string]$EnginePath = ".\target\release\vamender.exe",
    [string]$HostPath = ".\target\release\vamender-host.exe",
    [string]$PluginVarPath = ".\dist\AgenticCreator.VaMender.2.var",
    [string]$VaMRoot = "C:\Users\trist\_\VAM\VaMender-ReleaseTest",
    [string]$EvidenceRoot = "",
    [switch]$LaunchVaM,
    [int]$VaMStartupSeconds = 30,
    [switch]$KeepArtifacts
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
$markerName = "VAMENDER-ISOLATED-TEST-ENVIRONMENT.txt"

function Assert-True {
    param([bool]$Condition, [string]$Message)
    if (-not $Condition) {
        throw $Message
    }
}

function Write-RegressionTrace {
    param([string]$Message)

    "$(Get-Date -Format o) $Message" |
        Add-Content -LiteralPath (Join-Path $EvidenceRoot "regression-progress.log")
}

function Wait-ForFile {
    param(
        [string]$Path,
        [int]$TimeoutSeconds = 20
    )

    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    while ([DateTime]::UtcNow -lt $deadline) {
        if (Test-Path -LiteralPath $Path -PathType Leaf) {
            return
        }
        Start-Sleep -Milliseconds 200
    }
    throw "Timed out waiting for $Path"
}

function Invoke-Engine {
    param(
        [string]$Label,
        [string[]]$Arguments,
        [bool]$ExpectedSuccess = $true
    )

    $output = Join-Path $EvidenceRoot "engine-$Label.txt"
    $startInfo = [Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $EnginePath
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    foreach ($argument in $Arguments) {
        [void]$startInfo.ArgumentList.Add($argument)
    }
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $startInfo
    Assert-True $process.Start() "Could not start VaMender command '$Label'"
    if (-not $process.WaitForExit(60000)) {
        $process.Kill($true)
        $process.Dispose()
        throw "Engine command '$Label' did not exit within 60 seconds"
    }
    $exitCode = $process.ExitCode
    "exit_code=$exitCode" | Set-Content -LiteralPath $output
    $process.Dispose()
    if ($ExpectedSuccess -and $exitCode -ne 0) {
        throw "Engine command '$Label' failed with exit code $exitCode. See $output"
    }
    if (-not $ExpectedSuccess -and $exitCode -eq 0) {
        throw "Engine command '$Label' unexpectedly succeeded. See $output"
    }
    return [pscustomobject]@{ name = $Label; exit_code = $exitCode; output = $output }
}

function Assert-ReportSet {
    param([string]$Root)
    foreach ($name in @("actions_taken.txt", "actions_required.txt", "missing_dependencies.txt")) {
        Assert-True (Test-Path -LiteralPath (Join-Path $Root $name) -PathType Leaf) "Expected report is missing: $Root\\$name"
    }
}

$EnginePath = [IO.Path]::GetFullPath((Join-Path $projectRoot $EnginePath))
$HostPath = [IO.Path]::GetFullPath((Join-Path $projectRoot $HostPath))
$PluginVarPath = [IO.Path]::GetFullPath((Join-Path $projectRoot $PluginVarPath))
$VaMRoot = [IO.Path]::GetFullPath($VaMRoot).TrimEnd('\')
if ([string]::IsNullOrWhiteSpace($EvidenceRoot)) {
    $EvidenceRoot = Join-Path $VaMRoot (
        "VaMenderReleaseEvidence-" + (Get-Date -Format "yyyyMMdd-HHmmss")
    )
}
$EvidenceRoot = [IO.Path]::GetFullPath($EvidenceRoot)

foreach ($path in @($EnginePath, $HostPath, $PluginVarPath, (Join-Path $VaMRoot "VaM.exe"), (Join-Path $VaMRoot $markerName))) {
    Assert-True (Test-Path -LiteralPath $path -PathType Leaf) "Required isolated regression input is missing: $path"
}
New-Item -ItemType Directory -Force -Path $EvidenceRoot | Out-Null

$subsystemBytes = [IO.File]::ReadAllBytes($HostPath)
$peOffset = [BitConverter]::ToInt32($subsystemBytes, 0x3c)
$subsystem = [BitConverter]::ToUInt16($subsystemBytes, $peOffset + 24 + 68)
Assert-True ($subsystem -eq 2) "vamender-host.exe must use the Windows GUI subsystem; found $subsystem"

$corpusRoot = Join-Path $EvidenceRoot "synthetic-corpus"
Write-RegressionTrace "Starting synthetic VAR corpus."
& (Join-Path $PSScriptRoot "run-release-scenarios.ps1") -EnginePath $EnginePath -OutputRoot $corpusRoot -KeepArtifacts
if ($LASTEXITCODE -ne 0) {
    throw "The synthetic VAR corpus failed"
}

$packages = Join-Path $VaMRoot "AddonPackages"
$backup = Join-Path $EvidenceRoot "backup"
$fixtureSource = Join-Path $corpusRoot "01-read-only-inventory\AddonPackages"
Remove-Item -LiteralPath $packages -Recurse -Force
New-Item -ItemType Directory -Force -Path $packages | Out-Null
Copy-Item -Path (Join-Path $fixtureSource "*") -Destination $packages -Recurse -Force
$legacyPlugin = Join-Path $packages "AgenticCreator.VaMender.1.var"
Copy-Item -LiteralPath $PluginVarPath -Destination $legacyPlugin -Force
$legacyPluginHash = (Get-FileHash -LiteralPath $legacyPlugin -Algorithm SHA256).Hash.ToLowerInvariant()
Write-RegressionTrace "Copied clean fixture library into isolated VaM runtime."

$runKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
$runValueName = "VaMender"
$previousRunValue = $null
$hadRunValue = $false
try {
    $property = Get-ItemProperty -Path $runKey -Name $runValueName -ErrorAction Stop
    $previousRunValue = $property.$runValueName
    $hadRunValue = $true
} catch {
    $hadRunValue = $false
}

$previousLocalAppData = $env:LOCALAPPDATA
$isolatedLocalAppData = Join-Path $EvidenceRoot "LocalAppData"
$hostState = Join-Path $VaMRoot "Saves\PluginData\VaMender\Bridge"
$hostConfig = Join-Path $isolatedLocalAppData "VaMender\host.json"
$hostPid = $null
$vaMProcess = $null

try {
    $env:LOCALAPPDATA = $isolatedLocalAppData
    Write-RegressionTrace "Installing isolated host."
    Invoke-Engine -Label "install-host" -Arguments @("install-host", $VaMRoot, "--backup", $backup, "--plugin-var", $PluginVarPath) | Out-Null
    Write-RegressionTrace "Host installer exited; waiting for bridge state."
    Wait-ForFile (Join-Path $hostState "heartbeat.txt")
    Wait-ForFile (Join-Path $hostState "bridge.lock")
    Assert-True (Test-Path -LiteralPath $hostConfig -PathType Leaf) "Isolated host configuration was not created"
    $configuration = Get-Content -LiteralPath $hostConfig -Raw | ConvertFrom-Json
    Assert-True ($configuration.hostExecutable.EndsWith("vamender-host.exe")) "Startup configuration does not use the GUI host"
    Assert-True (Test-Path -LiteralPath $configuration.hostExecutable -PathType Leaf) "Installed GUI host is missing"
    $runValue = (Get-ItemProperty -Path $runKey -Name $runValueName).$runValueName
    Assert-True ($runValue -eq ('"' + $configuration.hostExecutable + '"')) "Windows startup is not registered directly to the isolated GUI host"
    Assert-True (-not $runValue.Contains(" host ")) "Windows startup still contains console host arguments"
    $installedPlugin = Join-Path $packages "AgenticCreator.VaMender.2.var"
    Assert-True (Test-Path -LiteralPath $installedPlugin -PathType Leaf) "Setup path did not install the Session Plugin VAR"
    Assert-True ((Get-FileHash -LiteralPath $installedPlugin -Algorithm SHA256).Hash -eq (Get-FileHash -LiteralPath $PluginVarPath -Algorithm SHA256).Hash) "Installed Session Plugin VAR checksum differs from the packaged artifact"
    Assert-True (-not (Test-Path -LiteralPath $legacyPlugin -PathType Leaf)) "Setup path did not retire the previous Session Plugin revision"
    $legacyBackup = Join-Path $backup ("install-history\$legacyPluginHash-AgenticCreator.VaMender.1.var")
    Assert-True (Test-Path -LiteralPath $legacyBackup -PathType Leaf) "Setup path did not preserve the previous Session Plugin revision"

    Write-RegressionTrace "Host installation verified; issuing bridge check."
    '{"id":"2001","operation":"check","deep":true}' | Set-Content -LiteralPath (Join-Path $hostState "request.json")
    Wait-ForFile (Join-Path $hostState "response.json")
    $response = Get-Content -LiteralPath (Join-Path $hostState "response.json") -Raw | ConvertFrom-Json
    Assert-True $response.success "Installed GUI host did not complete a bridge request"
    Assert-ReportSet (Join-Path $hostState "reports\2001\check")
    Write-RegressionTrace "Bridge check completed."

    if ($LaunchVaM) {
        $vaMProcess = Start-Process -FilePath (Join-Path $VaMRoot "VaM.exe") -WorkingDirectory $VaMRoot -PassThru
        Start-Sleep -Seconds $VaMStartupSeconds
        Assert-True (-not $vaMProcess.HasExited) "The isolated VaM runtime exited during startup"
    }

    Invoke-Engine -Label "stop-host" -Arguments @("stop-host") | Out-Null
    Write-RegressionTrace "Stop-host command exited; waiting for bridge lock removal."
    $deadline = [DateTime]::UtcNow.AddSeconds(20)
    while ((Test-Path -LiteralPath (Join-Path $hostState "bridge.lock") -PathType Leaf) -and [DateTime]::UtcNow -lt $deadline) {
        Start-Sleep -Milliseconds 200
    }
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $hostState "bridge.lock") -PathType Leaf)) "GUI tray host did not stop cooperatively"
    Invoke-Engine -Label "uninstall-host" -Arguments @("uninstall-host", "--purge") | Out-Null
    Assert-True (-not (Test-Path -LiteralPath $hostConfig -PathType Leaf)) "Isolated host configuration remained after purge"
    Assert-True (Test-Path -LiteralPath $backup -PathType Container) "Uninstall removed the isolated backup evidence"
    Write-RegressionTrace "Isolated host uninstall verified."
} finally {
    if ($null -ne $vaMProcess -and -not $vaMProcess.HasExited) {
        Stop-Process -Id $vaMProcess.Id -Force
    }
    if ($env:LOCALAPPDATA -eq $isolatedLocalAppData -and
        (Test-Path -LiteralPath $hostConfig -PathType Leaf)) {
        try {
            Invoke-Engine -Label "cleanup-stop-host" -Arguments @("stop-host") | Out-Null
            Invoke-Engine -Label "cleanup-uninstall-host" -Arguments @("uninstall-host", "--purge") | Out-Null
        } catch {
            Write-Warning "Could not fully clean up the isolated VaMender host: $($_.Exception.Message)"
        }
    }
    if ($null -ne $previousLocalAppData) {
        $env:LOCALAPPDATA = $previousLocalAppData
    } else {
        Remove-Item Env:LOCALAPPDATA -ErrorAction SilentlyContinue
    }
    if ($hadRunValue) {
        New-ItemProperty -Path $runKey -Name $runValueName -PropertyType String -Value $previousRunValue -Force | Out-Null
    } else {
        Remove-ItemProperty -Path $runKey -Name $runValueName -ErrorAction SilentlyContinue
    }
}

$summary = [pscustomobject]@{
    isolated_vam_root = $VaMRoot
    evidence_root = $EvidenceRoot
    gui_subsystem = $subsystem
    startup_registration_restored = $true
    synthetic_corpus = Join-Path $corpusRoot "summary.json"
    host_state = $hostState
    generated_at_utc = [DateTime]::UtcNow.ToString("o")
}
$summary | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $EvidenceRoot "isolated-regression-summary.json")
$summary | ConvertTo-Json -Depth 8

if (-not $KeepArtifacts) {
    Write-Output "Isolated VaM and synthetic evidence are retained for review; remove only the marked test environment when no longer needed."
}

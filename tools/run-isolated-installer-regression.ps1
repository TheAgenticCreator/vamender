# SPDX-License-Identifier: MIT

[CmdletBinding()]
param(
    [string]$SetupPath = "",
    [string]$PluginVarPath = ".\dist\AgenticCreator.VaMender.2.var",
    [string]$VaMRoot = "C:\Users\trist\_\VAM\VaMender-ReleaseTest",
    [string]$EvidenceRoot = ""
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

function Wait-ForFile {
    param([string]$Path, [int]$TimeoutSeconds = 30)

    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    while ([DateTime]::UtcNow -lt $deadline) {
        if (Test-Path -LiteralPath $Path -PathType Leaf) {
            return
        }
        Start-Sleep -Milliseconds 200
    }
    throw "Timed out waiting for $Path"
}

function Invoke-NativeProcess {
    param(
        [string]$Label,
        [string]$FilePath,
        [string[]]$Arguments,
        [int]$TimeoutSeconds = 120
    )

    $startInfo = [Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $FilePath
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    foreach ($argument in $Arguments) {
        [void]$startInfo.ArgumentList.Add($argument)
    }
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $startInfo
    Assert-True $process.Start() "Could not start $Label"
    if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
        $process.Kill($true)
        $process.Dispose()
        throw "$Label did not exit within $TimeoutSeconds seconds"
    }
    $exitCode = $process.ExitCode
    $process.Dispose()
    if ($exitCode -ne 0) {
        throw "$Label failed with exit code $exitCode"
    }
}

function Resolve-ProjectPath {
    param([string]$Path)

    if ([IO.Path]::IsPathRooted($Path)) {
        return [IO.Path]::GetFullPath($Path)
    }
    return [IO.Path]::GetFullPath((Join-Path $projectRoot $Path))
}

$expectedVersion = (
    Select-String -Path (Join-Path $projectRoot "Cargo.toml") -Pattern '^version = "([^"]+)"'
).Matches[0].Groups[1].Value
if ([string]::IsNullOrWhiteSpace($SetupPath)) {
    $SetupPath = ".\dist\VaMender-Setup-$expectedVersion.exe"
}
$SetupPath = Resolve-ProjectPath $SetupPath
$PluginVarPath = Resolve-ProjectPath $PluginVarPath
$VaMRoot = [IO.Path]::GetFullPath($VaMRoot).TrimEnd('\')
if ([string]::IsNullOrWhiteSpace($EvidenceRoot)) {
    $EvidenceRoot = Join-Path $VaMRoot (
        "VaMenderInstallerEvidence-" + (Get-Date -Format "yyyyMMdd-HHmmss")
    )
}
$EvidenceRoot = [IO.Path]::GetFullPath($EvidenceRoot)

foreach ($path in @($SetupPath, $PluginVarPath, (Join-Path $VaMRoot "VaM.exe"), (Join-Path $VaMRoot $markerName))) {
    Assert-True (Test-Path -LiteralPath $path -PathType Leaf) "Required isolated installer input is missing: $path"
}

$setupVersion = (Get-Item -LiteralPath $SetupPath).VersionInfo.ProductVersion.Trim()
Assert-True ($setupVersion -eq $expectedVersion) "Expected a $expectedVersion Setup candidate, found $setupVersion"
New-Item -ItemType Directory -Force -Path $EvidenceRoot | Out-Null

$externalHost = Get-CimInstance Win32_Process |
    Where-Object {
        $_.Name -in @("vamender.exe", "vamender-host.exe") -and
        $_.ExecutablePath -and
        -not $_.ExecutablePath.StartsWith($EvidenceRoot, [StringComparison]::OrdinalIgnoreCase)
    }
Assert-True ($null -eq $externalHost) "Exit any normal VaMender tray host before running the installer regression."

$packages = Join-Path $VaMRoot "AddonPackages"
$backup = Join-Path $EvidenceRoot "backup"
$installerApp = Join-Path $EvidenceRoot "InstallerApp"
$installerLocalAppData = Join-Path $EvidenceRoot "InstallerLocalAppData"
$hostState = Join-Path $VaMRoot "Saves\PluginData\VaMender\Bridge"
$hostConfig = Join-Path $installerLocalAppData "VaMender\host.json"
$legacyPlugin = Join-Path $packages "AgenticCreator.VaMender.1.var"
$installedPlugin = Join-Path $packages "AgenticCreator.VaMender.2.var"

Remove-Item -LiteralPath $legacyPlugin -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $installedPlugin -Force -ErrorAction SilentlyContinue
Copy-Item -LiteralPath $PluginVarPath -Destination $legacyPlugin -Force
$legacyHash = (Get-FileHash -LiteralPath $legacyPlugin -Algorithm SHA256).Hash.ToLowerInvariant()

$runKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
$runValueName = "VaMender"
$previousRunValue = $null
$hadRunValue = $false
try {
    $previousRunValue = (Get-ItemProperty -Path $runKey -Name $runValueName -ErrorAction Stop).$runValueName
    $hadRunValue = $true
} catch {
    $hadRunValue = $false
}
$previousLocalAppData = $env:LOCALAPPDATA

try {
    $env:LOCALAPPDATA = $installerLocalAppData
    Invoke-NativeProcess -Label "isolated Setup install" -FilePath $SetupPath -Arguments @(
        "/VERYSILENT",
        "/SUPPRESSMSGBOXES",
        "/NORESTART",
        "/DIR=$installerApp",
        "/VAMROOT=$VaMRoot",
        "/BACKUP=$backup"
    )

    Assert-True (Test-Path -LiteralPath (Join-Path $installerApp "vamender.exe") -PathType Leaf) "Setup did not install vamender.exe"
    Assert-True (Test-Path -LiteralPath (Join-Path $installerApp "vamender-host.exe") -PathType Leaf) "Setup did not install the GUI host"
    Assert-True (Test-Path -LiteralPath $hostConfig -PathType Leaf) "Setup did not create isolated host configuration"
    Wait-ForFile (Join-Path $hostState "heartbeat.txt")
    Wait-ForFile (Join-Path $hostState "bridge.lock")
    $configuration = Get-Content -LiteralPath $hostConfig -Raw | ConvertFrom-Json
    $runValue = (Get-ItemProperty -Path $runKey -Name $runValueName).$runValueName
    Assert-True ($runValue -eq ('"' + $configuration.hostExecutable + '"')) "Setup did not register the GUI host directly"
    Assert-True (-not $runValue.Contains(" host ")) "Setup registered console host arguments"
    Assert-True (Test-Path -LiteralPath $installedPlugin -PathType Leaf) "Setup did not install plugin revision 2"
    Assert-True (-not (Test-Path -LiteralPath $legacyPlugin -PathType Leaf)) "Setup did not retire plugin revision 1"
    $legacyBackup = Join-Path $backup ("install-history\$legacyHash-AgenticCreator.VaMender.1.var")
    Assert-True (Test-Path -LiteralPath $legacyBackup -PathType Leaf) "Setup did not preserve plugin revision 1"

    '{"id":"2002","operation":"check","deep":true}' | Set-Content -LiteralPath (Join-Path $hostState "request.json")
    Wait-ForFile (Join-Path $hostState "response.json")
    $response = Get-Content -LiteralPath (Join-Path $hostState "response.json") -Raw | ConvertFrom-Json
    Assert-True $response.success "Installer-started host did not complete a bridge request"

    $uninstaller = Join-Path $installerApp "unins000.exe"
    Assert-True (Test-Path -LiteralPath $uninstaller -PathType Leaf) "Setup did not create an uninstaller"
    Invoke-NativeProcess -Label "isolated Setup uninstall" -FilePath $uninstaller -Arguments @(
        "/VERYSILENT",
        "/SUPPRESSMSGBOXES",
        "/NORESTART"
    )
    Assert-True (-not (Test-Path -LiteralPath $hostConfig -PathType Leaf)) "Uninstaller left isolated host configuration behind"
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $installerApp "vamender.exe") -PathType Leaf)) "Uninstaller left the engine behind"
    Assert-True (Test-Path -LiteralPath $installedPlugin -PathType Leaf) "Uninstaller removed the installed Session Plugin VAR"
    Assert-True (Test-Path -LiteralPath $legacyBackup -PathType Leaf) "Uninstaller removed plugin backup evidence"
} finally {
    if ($env:LOCALAPPDATA -eq $installerLocalAppData -and (Test-Path -LiteralPath $hostConfig -PathType Leaf)) {
        $installedEngine = Join-Path $installerApp "vamender.exe"
        if (Test-Path -LiteralPath $installedEngine -PathType Leaf) {
            try {
                Invoke-NativeProcess -Label "installer cleanup" -FilePath $installedEngine -Arguments @("uninstall-host", "--purge")
            } catch {
                Write-Warning "Could not fully clean up the isolated installer host: $($_.Exception.Message)"
            }
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
    setup = $SetupPath
    isolated_vam_root = $VaMRoot
    evidence_root = $EvidenceRoot
    startup_registration_restored = $true
    plugin_revision_upgrade_verified = $true
    generated_at_utc = [DateTime]::UtcNow.ToString("o")
}
$summary | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $EvidenceRoot "isolated-installer-regression-summary.json")
$summary | ConvertTo-Json -Depth 8

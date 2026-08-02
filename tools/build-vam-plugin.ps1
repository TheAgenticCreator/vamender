# SPDX-License-Identifier: MIT

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$VaMPath,
    [string]$OutputPath =
        ".\vam-plugin\Custom\Scripts\AgenticCreator\VaMender\VaMender.dll"
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
$managed = Join-Path ([IO.Path]::GetFullPath($VaMPath)) "VaM_Data\Managed"
$assemblyCSharp = Join-Path $managed "Assembly-CSharp.dll"
if (-not (Test-Path -LiteralPath $assemblyCSharp -PathType Leaf)) {
    throw "VaM Assembly-CSharp.dll was not found below: $managed"
}

$compilerCandidates = @(
    "$env:WINDIR\Microsoft.NET\Framework64\v3.5\csc.exe",
    "$env:WINDIR\Microsoft.NET\Framework\v3.5\csc.exe"
)
$compiler = $compilerCandidates |
    Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } |
    Select-Object -First 1
if ($null -eq $compiler) {
    throw (
        ".NET Framework 3.5 csc.exe is required. VaM uses CLR 2; " +
        "do not build its plugin with the CLR 4 compiler."
    )
}

$sourceRoot = Join-Path $projectRoot (
    "vam-plugin\Custom\Scripts\AgenticCreator\VaMender\src"
)
$sources = @(
    Get-ChildItem -LiteralPath $sourceRoot -Filter "*.cs" -File |
        Sort-Object Name |
        ForEach-Object FullName
)
if ($sources.Count -eq 0) {
    throw "No VaM plugin sources were found below: $sourceRoot"
}

$resolvedOutput = if ([IO.Path]::IsPathRooted($OutputPath)) {
    [IO.Path]::GetFullPath($OutputPath)
} else {
    [IO.Path]::GetFullPath((Join-Path $projectRoot $OutputPath))
}
New-Item -ItemType Directory -Force -Path (
    Split-Path -Parent $resolvedOutput
) | Out-Null

$references = @(
    "Assembly-CSharp.dll",
    "UnityEngine.dll",
    "UnityEngine.CoreModule.dll",
    "UnityEngine.UI.dll"
)
$arguments = @(
    "/nologo",
    "/target:library",
    "/optimize+",
    "/warn:4",
    "/warnaserror+",
    "/out:$resolvedOutput"
)
foreach ($reference in $references) {
    $path = Join-Path $managed $reference
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Required VaM assembly is missing: $path"
    }
    $arguments += "/reference:$path"
}
$arguments += $sources
& $compiler $arguments
if ($LASTEXITCODE -ne 0) {
    throw "VaM CLR 2 plugin compilation failed with exit code $LASTEXITCODE"
}

$validationRoot = Join-Path $projectRoot "target\plugin-validation"
New-Item -ItemType Directory -Force -Path $validationRoot | Out-Null
$loaderSource = Join-Path $projectRoot (
    "vam-plugin\test\VaMTypeLoadValidation.cs"
)
$loader = Join-Path $validationRoot "VaMTypeLoadValidation.exe"
& $compiler /nologo /target:exe /optimize+ /warn:4 /warnaserror+ `
    "/out:$loader" $loaderSource
if ($LASTEXITCODE -ne 0) {
    throw "VaM type-load validator compilation failed"
}
& $loader $managed $resolvedOutput
if ($LASTEXITCODE -ne 0) {
    throw "VaM CLR 2 Assembly.GetTypes validation failed"
}

dotnet build `
    (Join-Path $projectRoot "vam-plugin\test\VaMender.Plugin.Validation.csproj") `
    --configuration Release --no-restore
if ($LASTEXITCODE -ne 0) {
    throw "VaM metadata validator compilation failed"
}
dotnet run `
    --project (
        Join-Path $projectRoot "vam-plugin\test\VaMender.Plugin.Validation.csproj"
    ) `
    --configuration Release --no-build -- $resolvedOutput
if ($LASTEXITCODE -ne 0) {
    throw "VaM sandbox metadata validation failed"
}

$hash = (
    Get-FileHash -LiteralPath $resolvedOutput -Algorithm SHA256
).Hash.ToLowerInvariant()
Write-Output "Created CLR 2 VaM plugin: $resolvedOutput"
Write-Output "SHA-256 $hash"

# SPDX-License-Identifier: MIT

[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$expectedName = "TheAgenticCreator"
$expectedEmail = "312204356+TheAgenticCreatorDev@users.noreply.github.com"

$repositoryRoot = (& git rev-parse --show-toplevel).Trim()
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($repositoryRoot)) {
    throw "Run this command inside the VaMender Git repository."
}

Push-Location -LiteralPath $repositoryRoot
try {
    & git config --local user.name $expectedName
    & git config --local user.email $expectedEmail
    & git config --local core.hooksPath .githooks
    if ($LASTEXITCODE -ne 0) {
        throw "Could not configure the repository-local maintainer identity."
    }

    & "$PSScriptRoot/check-maintainer-identity.ps1" -Revision HEAD
    if ($LASTEXITCODE -ne 0) {
        throw "Existing history does not satisfy the VaMender identity policy."
    }
} finally {
    Pop-Location
}

Write-Host "Repository-local identity: $expectedName <$expectedEmail>"
Write-Host "Pre-push guard: .githooks/pre-push"
Write-Host "Authenticate GitHub operations as TheAgenticCreatorDev or an approved organization App."

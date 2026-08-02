# SPDX-License-Identifier: MIT

[CmdletBinding()]
param(
    [Parameter()]
    [ValidateNotNullOrEmpty()]
    [string]$Revision = "HEAD",

    [Parameter()]
    [string]$Tag
)

$ErrorActionPreference = "Stop"
$expectedName = "TheAgenticCreator"
$expectedEmail = "312204356+TheAgenticCreatorDev@users.noreply.github.com"
$separator = [char]0x1f
$format = "%H%x1f%an%x1f%ae%x1f%cn%x1f%ce"

$records = @(& git log "--format=$format" $Revision)
if ($LASTEXITCODE -ne 0) {
    throw "Cannot inspect Git revision '$Revision'."
}
if ($records.Count -eq 0) {
    throw "Revision '$Revision' contains no commits to validate."
}

$failures = [System.Collections.Generic.List[string]]::new()
foreach ($record in $records) {
    $fields = $record.Split($separator)
    if ($fields.Count -ne 5) {
        throw "Unexpected Git identity record format."
    }
    $commit, $authorName, $authorEmail, $committerName, $committerEmail = $fields
    if ($authorName -ne $expectedName -or $authorEmail -ne $expectedEmail) {
        $failures.Add("$commit has author '$authorName <$authorEmail>'.")
    }
    if ($committerName -ne $expectedName -or $committerEmail -ne $expectedEmail) {
        $failures.Add("$commit has committer '$committerName <$committerEmail>'.")
    }
}

if (-not [string]::IsNullOrWhiteSpace($Tag)) {
    $type = (& git cat-file -t $Tag).Trim()
    if ($LASTEXITCODE -ne 0 -or $type -ne "tag") {
        $failures.Add("Release ref '$Tag' must be an annotated tag.")
    } else {
        $tagger = (& git for-each-ref "--format=%(taggername)%1f%(taggeremail)" "refs/tags/$Tag").Trim()
        $taggerFields = $tagger.Split($separator)
        $taggerName = if ($taggerFields.Count -gt 0) { $taggerFields[0] } else { "" }
        $taggerEmail = if ($taggerFields.Count -gt 1) {
            $taggerFields[1].Trim().TrimStart('<').TrimEnd('>')
        } else {
            ""
        }
        if ($taggerName -ne $expectedName -or $taggerEmail -ne $expectedEmail) {
            $failures.Add("Tag '$Tag' has tagger '$taggerName <$taggerEmail>'.")
        }
    }
}

if ($failures.Count -gt 0) {
    $failures | ForEach-Object { [Console]::Error.WriteLine("ERROR: $_") }
    throw "Maintainer identity validation failed. Run tools/configure-maintainer-identity.ps1 and rewrite the affected commits or tag."
}

Write-Host "Validated $($records.Count) commit(s) as $expectedName <$expectedEmail>."
if (-not [string]::IsNullOrWhiteSpace($Tag)) {
    Write-Host "Validated annotated tag '$Tag'."
}

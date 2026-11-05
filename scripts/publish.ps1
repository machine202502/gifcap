# Package the portable zip and the installer, then attach both to a GitHub release tag.
param(
    [string]$Tag
)

$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $Root

$cargo = Get-Content -Raw -Path (Join-Path $Root "Cargo.toml")
if ($cargo -notmatch '(?m)^version\s*=\s*"([^"]+)"') {
    throw "Could not read the package version from Cargo.toml"
}
$Version = $Matches[1]
if (-not $Tag) { $Tag = "v$Version" }

& (Join-Path $PSScriptRoot "package.ps1") -Version $Version
if ($LASTEXITCODE -ne 0) { throw "package.ps1 failed" }

$portable = Join-Path $Root "dist\gifcap-$Version-windows-x64.zip"
$setup = Join-Path $Root "dist\gifcap-$Version-windows-x64-setup.exe"
if (-not (Test-Path $portable)) { throw "Missing release file: $portable" }
if (-not (Test-Path $setup)) { throw "Missing release file: $setup" }

$notes = @"
Windows x64.

``gifcap-$Version-windows-x64-setup.exe`` installs the program.
``gifcap-$Version-windows-x64.zip`` is portable. Unzip it and run ``gifcap.exe``.

The executable records GIF only. It includes FFmpeg under the GNU LGPL v2.1 or later. ``COPYING``, ``NOTICE``, and ``THIRD-PARTY.txt`` travel with both files.
gifcap source for this tag: https://github.com/machine202502/gifcap/tree/$Tag
"@

gh release create $Tag --title "gifcap $Version" --notes $notes $setup $portable
if ($LASTEXITCODE -ne 0) {
    throw "gh release create failed. If $Tag already exists, delete that release before publishing again."
}

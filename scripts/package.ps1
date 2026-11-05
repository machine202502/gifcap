# Build the release exe, then write the portable zip and the installer into dist/.
# Requires FFMPEG_DIR (static FFmpeg prefix) and Inno Setup 6 (ISCC.exe).
param(
    [string]$Version
)

$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $Root

if (-not $Version) {
    $cargo = Get-Content -Raw -Path (Join-Path $Root "Cargo.toml")
    if ($cargo -notmatch '(?m)^version\s*=\s*"([^"]+)"') {
        throw "Could not read the package version from Cargo.toml"
    }
    $Version = $Matches[1]
}

if (-not $env:FFMPEG_DIR) {
    throw "Set FFMPEG_DIR to the static FFmpeg prefix before packaging."
}

Write-Host "Building gifcap $Version"
cargo build --release -p gifcap
if ($LASTEXITCODE -ne 0) {
    throw "cargo build --release failed"
}

$exe = Join-Path $Root "target\x86_64-pc-windows-msvc\release\gifcap.exe"
if (-not (Test-Path $exe)) {
    $exe = Join-Path $Root "target\release\gifcap.exe"
}
if (-not (Test-Path $exe)) {
    throw "Release executable was not produced"
}

$dist = Join-Path $Root "dist"
New-Item -ItemType Directory -Force -Path $dist | Out-Null

$lines = New-Object System.Collections.Generic.List[string]
$lines.Add("gifcap $Version")
$lines.Add("")
$lines.Add("This executable includes FFmpeg under the GNU LGPL v2.1 or later. See COPYING and NOTICE.")
$lines.Add("Source: https://github.com/machine202502/gifcap/tree/v$Version")
$lines.Add("")
$lines.Add("==== Native libraries ====")
$share = Join-Path $env:FFMPEG_DIR "share"
if (Test-Path $share) {
    foreach ($dir in (Get-ChildItem -Directory $share | Sort-Object Name)) {
        $copy = Join-Path $dir.FullName "copyright"
        if (-not (Test-Path $copy)) { continue }
        $lines.Add("")
        $lines.Add("---- $($dir.Name) ----")
        $lines.Add([System.IO.File]::ReadAllText($copy).TrimEnd())
    }
}
$lines.Add("")
$lines.Add("==== Rust crates ====")
Push-Location $Root
try {
    $metaJson = (& cargo metadata --format-version 1 | Out-String)
} finally {
    Pop-Location
}
if ($metaJson) {
    if ($PSVersionTable.PSVersion.Major -ge 6) {
        $meta = $metaJson | ConvertFrom-Json -Depth 20
    } else {
        $meta = $metaJson | ConvertFrom-Json
    }
    foreach ($pkg in ($meta.packages | Where-Object { $_.source } | Sort-Object name)) {
        $authors = @($pkg.authors) -join ", "
        if (-not $authors) { $authors = "not recorded" }
        $lines.Add("$($pkg.name) $($pkg.version) — $($pkg.license) — $authors")
    }
}

$thirdParty = Join-Path $dist "THIRD-PARTY.txt"
[System.IO.File]::WriteAllLines($thirdParty, $lines)

$stage = Join-Path $dist "portable"
if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
New-Item -ItemType Directory -Force -Path $stage | Out-Null
Copy-Item $exe (Join-Path $stage "gifcap.exe")
Copy-Item (Join-Path $Root "COPYING") (Join-Path $stage "COPYING")
Copy-Item (Join-Path $Root "NOTICE") (Join-Path $stage "NOTICE")
Copy-Item $thirdParty (Join-Path $stage "THIRD-PARTY.txt")
$portable = Join-Path $dist "gifcap-$Version-windows-x64.zip"
if (Test-Path $portable) { Remove-Item $portable }
Compress-Archive -Path (Join-Path $stage "*") -DestinationPath $portable
Remove-Item $stage -Recurse -Force
Write-Host "Portable: $portable"

$iscc = @(
    "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe",
    "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
    "$env:ProgramFiles\Inno Setup 6\ISCC.exe"
) | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1
if (-not $iscc) {
    throw "Inno Setup 6 was not found. Install it so ISCC.exe can build the installer."
}

$iss = Join-Path $Root "installer\gifcap.iss"
& $iscc "/DAppVersion=$Version" "/DSourceExe=$exe" "/DThirdParty=$thirdParty" $iss
if ($LASTEXITCODE -ne 0) {
    throw "ISCC failed"
}

$setup = Join-Path $dist "gifcap-$Version-windows-x64-setup.exe"
if (-not (Test-Path $setup)) {
    throw "Installer was not produced: $setup"
}
Write-Host "Installer: $setup"

# CRONUS installer for Windows (x86_64): downloads the prebuilt cronus.exe from
# GitHub Releases, verifies its SHA256 and installs it for the current user.
#
#   irm https://raw.githubusercontent.com/pedrogbraz/cronus-kernel/main/scripts/install.ps1 | iex
#
# Environment:
#   CRONUS_VERSION           release to install, e.g. 0.2.0 or v0.2.0 (default: latest)
#   CRONUS_REPO              GitHub owner/repo publishing releases (default: pedrogbraz/cronus-kernel)
#   CRONUS_INSTALL_DIR       install directory (default: $HOME\.cronus\bin)
#   CRONUS_INSTALL_DRY_RUN=1 print what would be downloaded and exit (no network)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$repo = if ($env:CRONUS_REPO) { $env:CRONUS_REPO } else { 'pedrogbraz/cronus-kernel' }
$target = 'x86_64-pc-windows-msvc'
$installDir = if ($env:CRONUS_INSTALL_DIR) { $env:CRONUS_INSTALL_DIR } else { Join-Path $HOME '.cronus\bin' }
$version = if ($env:CRONUS_VERSION) { $env:CRONUS_VERSION.TrimStart('v') } else { '' }

if (-not [Environment]::Is64BitOperatingSystem) {
    throw 'cronus-install: only 64-bit Windows is supported'
}

if ($env:CRONUS_INSTALL_DRY_RUN -in @('1', 'true', 'yes')) {
    $shown = if ($version) { $version } else { 'latest' }
    $base = "https://github.com/$repo/releases/download/v$shown"
    Write-Output 'cronus-install: dry run, nothing downloaded'
    Write-Output "repo=$repo"
    Write-Output "version=$shown"
    Write-Output "target=$target"
    Write-Output "asset_url=$base/cronus-$shown-$target.zip"
    Write-Output "checksums_url=$base/SHA256SUMS"
    Write-Output "install_path=$(Join-Path $installDir 'cronus.exe')"
    return
}

[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

if (-not $version) {
    $release = Invoke-RestMethod -UseBasicParsing "https://api.github.com/repos/$repo/releases/latest"
    $version = $release.tag_name.TrimStart('v')
}

$asset = "cronus-$version-$target.zip"
$base = "https://github.com/$repo/releases/download/v$version"
$tmp = Join-Path ([IO.Path]::GetTempPath()) ("cronus-install-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $tmp | Out-Null

try {
    Write-Output "Downloading cronus $version for $target..."
    $zip = Join-Path $tmp $asset
    $sums = Join-Path $tmp 'SHA256SUMS'
    Invoke-WebRequest -UseBasicParsing -Uri "$base/$asset" -OutFile $zip
    Invoke-WebRequest -UseBasicParsing -Uri "$base/SHA256SUMS" -OutFile $sums

    $line = Get-Content $sums | Where-Object { ($_ -split '\s+')[1] -in @($asset, "*$asset") } | Select-Object -First 1
    if (-not $line) { throw "cronus-install: $asset is not listed in SHA256SUMS" }
    $expected = ($line -split '\s+')[0].ToLowerInvariant()
    $actual = (Get-FileHash -Algorithm SHA256 -Path $zip).Hash.ToLowerInvariant()
    if ($expected -ne $actual) { throw "cronus-install: checksum mismatch for $asset (expected $expected, got $actual)" }

    Expand-Archive -Path $zip -DestinationPath $tmp -Force
    $exe = Join-Path $tmp "cronus-$version-$target\cronus.exe"
    if (-not (Test-Path $exe)) { throw "cronus-install: archive $asset does not contain cronus.exe" }

    New-Item -ItemType Directory -Force -Path $installDir | Out-Null
    Copy-Item $exe (Join-Path $installDir 'cronus.exe') -Force
}
finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}

Write-Output "Installed $(& (Join-Path $installDir 'cronus.exe') --version) to $installDir"

$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (-not (($userPath -split ';') -contains $installDir)) {
    Write-Output ''
    Write-Output 'cronus is not on your PATH yet. Add it for your user with:'
    Write-Output "  [Environment]::SetEnvironmentVariable('Path', `"$installDir;`" + [Environment]::GetEnvironmentVariable('Path', 'User'), 'User')"
    Write-Output 'Then open a new terminal.'
}

#!/bin/sh
# CRONUS installer: downloads a prebuilt `cronus` binary from GitHub Releases,
# verifies its SHA256 and installs it without sudo.
#
#   curl -fsSL https://raw.githubusercontent.com/pedrogbraz/cronus-kernel/main/scripts/install.sh | sh
#
# Environment:
#   CRONUS_VERSION           release to install, e.g. 0.2.0 or v0.2.0 (default: latest)
#   CRONUS_REPO              GitHub owner/repo publishing releases (default: pedrogbraz/cronus-kernel)
#   CRONUS_INSTALL_DIR       install directory (default: $HOME/.cronus/bin)
#   CRONUS_TARGET            force a release target triple (skips OS/arch detection)
#   CRONUS_INSTALL_DRY_RUN=1 print what would be downloaded and exit (no network)
#   CRONUS_OS, CRONUS_ARCH   override `uname -s` / `uname -m` (testing)
set -eu

REPO="${CRONUS_REPO:-pedrogbraz/cronus-kernel}"

say() { printf '%s\n' "$*"; }
err() {
  printf 'cronus-install: error: %s\n' "$*" >&2
  exit 1
}
has() { command -v "$1" >/dev/null 2>&1; }

detect_target() {
  if [ -n "${CRONUS_TARGET:-}" ]; then
    printf '%s' "$CRONUS_TARGET"
    return
  fi
  os="${CRONUS_OS:-$(uname -s)}"
  arch="${CRONUS_ARCH:-$(uname -m)}"
  case "$os" in
    Darwin)
      # A shell under Rosetta reports x86_64 on Apple silicon; prefer the native build.
      if [ -z "${CRONUS_ARCH:-}" ] && [ "$arch" = x86_64 ] &&
        [ "$(sysctl -n sysctl.proc_translated 2>/dev/null || echo 0)" = 1 ]; then
        arch=arm64
      fi
      case "$arch" in
        arm64 | aarch64) printf 'aarch64-apple-darwin' ;;
        x86_64 | amd64) printf 'x86_64-apple-darwin' ;;
        *) err "unsupported macOS architecture: $arch" ;;
      esac
      ;;
    Linux)
      case "$arch" in
        x86_64 | amd64) printf 'x86_64-unknown-linux-musl' ;;
        *) err "no prebuilt Linux binary for $arch; build from source: cargo install --git https://github.com/$REPO --locked" ;;
      esac
      ;;
    MINGW* | MSYS* | CYGWIN* | Windows_NT)
      err "on Windows use scripts/install.ps1 (PowerShell) or download the .zip from https://github.com/$REPO/releases"
      ;;
    *) err "unsupported operating system: $os" ;;
  esac
}

fetch() {
  if has curl; then
    curl --proto '=https' --tlsv1.2 -fsSL "$1" -o "$2"
  elif has wget; then
    wget -q -O "$2" "$1"
  else
    err "curl or wget is required"
  fi
}

resolve_latest() {
  if has curl; then
    # /releases/latest redirects to /releases/tag/vX.Y.Z (no API rate limit).
    effective=$(curl --proto '=https' --tlsv1.2 -fsSLI -o /dev/null -w '%{url_effective}' \
      "https://github.com/$REPO/releases/latest") || effective=""
    tag="${effective##*/}"
  elif has wget; then
    tag=$(wget -q -O - "https://api.github.com/repos/$REPO/releases/latest" |
      sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n 1) || tag=""
  else
    err "curl or wget is required"
  fi
  case "$tag" in
    v[0-9]*) printf '%s' "${tag#v}" ;;
    *) err "could not find a published release for $REPO (set CRONUS_VERSION)" ;;
  esac
}

sha256_of() {
  if has sha256sum; then
    sha256sum "$1" | awk '{print $1}'
  elif has shasum; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    err "sha256sum or shasum is required to verify the download"
  fi
}

target=$(detect_target)
version="${CRONUS_VERSION:-}"
version="${version#v}"
if [ -n "${CRONUS_INSTALL_DIR:-}" ]; then
  install_dir="$CRONUS_INSTALL_DIR"
else
  [ -n "${HOME:-}" ] || err "HOME is not set; set CRONUS_INSTALL_DIR"
  install_dir="$HOME/.cronus/bin"
fi

case "${CRONUS_INSTALL_DRY_RUN:-0}" in
  1 | true | yes)
    shown="${version:-latest}"
    base="https://github.com/$REPO/releases/download/v$shown"
    say "cronus-install: dry run, nothing downloaded"
    say "repo=$REPO"
    say "version=$shown"
    say "target=$target"
    say "asset_url=$base/cronus-$shown-$target.tar.gz"
    say "checksums_url=$base/SHA256SUMS"
    say "install_path=$install_dir/cronus"
    exit 0
    ;;
esac

has tar || err "tar is required"
[ -n "$version" ] || version=$(resolve_latest)
asset="cronus-$version-$target.tar.gz"
base="https://github.com/$REPO/releases/download/v$version"

tmp=$(mktemp -d 2>/dev/null || mktemp -d -t cronus-install)
trap 'rm -rf "$tmp"' EXIT
trap 'exit 130' INT TERM

say "Downloading cronus $version for $target..."
fetch "$base/$asset" "$tmp/$asset" ||
  err "download failed: $base/$asset (does release v$version exist for $target?)"
fetch "$base/SHA256SUMS" "$tmp/SHA256SUMS" || err "download failed: $base/SHA256SUMS"

expected=$(awk -v a="$asset" '$2 == a || $2 == "*" a { print $1; exit }' "$tmp/SHA256SUMS")
[ -n "$expected" ] || err "$asset is not listed in SHA256SUMS"
actual=$(sha256_of "$tmp/$asset")
[ "$expected" = "$actual" ] || err "checksum mismatch for $asset (expected $expected, got $actual)"

tar -xzf "$tmp/$asset" -C "$tmp"
bin="$tmp/cronus-$version-$target/cronus"
[ -f "$bin" ] || err "archive $asset does not contain cronus-$version-$target/cronus"

mkdir -p "$install_dir" || err "cannot create $install_dir (set CRONUS_INSTALL_DIR)"
cp "$bin" "$install_dir/.cronus.new.$$"
chmod 0755 "$install_dir/.cronus.new.$$"
mv -f "$install_dir/.cronus.new.$$" "$install_dir/cronus"

say "Installed $("$install_dir/cronus" --version 2>/dev/null || echo "cronus $version") to $install_dir/cronus"

case ":${PATH:-}:" in
  *":$install_dir:"*) ;;
  *)
    case "${SHELL:-}" in
      */zsh) rc="$HOME/.zshrc" ;;
      */bash) rc="$HOME/.bashrc" ;;
      *) rc="$HOME/.profile" ;;
    esac
    say ""
    say "cronus is not on your PATH yet. Add it with:"
    case "${SHELL:-}" in
      */fish) say "  fish_add_path $install_dir" ;;
      *) say "  echo 'export PATH=\"$install_dir:\$PATH\"' >> $rc && . $rc" ;;
    esac
    ;;
esac

#!/usr/bin/env bash
# Install the pinned linker binary and the repository's Rust toolchain.

set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=scripts/build-tools-common.sh
. "$script_dir/build-tools-common.sh"

MOLD_RELEASE_BASE_URL=${MOLD_RELEASE_BASE_URL:-https://github.com/rui314/mold/releases/download}
CURL_CONNECT_TIMEOUT=${CURL_CONNECT_TIMEOUT:-15}
CURL_MIN_BYTES_PER_SECOND=${CURL_MIN_BYTES_PER_SECOND:-1024}
CURL_STALL_SECONDS=${CURL_STALL_SECONDS:-60}

BUILD_TOOLS_WORKDIR=

remove_workdir() {
  [ -n "$BUILD_TOOLS_WORKDIR" ] || return 0
  rm -rf -- "$BUILD_TOOLS_WORKDIR"
  BUILD_TOOLS_WORKDIR=
}

trap remove_workdir EXIT

verify_mold_archive() {
  local archive=$1 name=$2 expected recorded
  expected=$(awk -v name="$name" '$2 == name { print $1 }' "$MOLD_SHA256SUMS_FILE")
  [ -n "$expected" ] || fail "no checksum recorded for $name in $MOLD_SHA256SUMS_FILE"
  recorded=$(printf '%s\n' "$expected" | grep -c .)
  [ "$recorded" -eq 1 ] ||
    fail "$recorded checksums recorded for $name in $MOLD_SHA256SUMS_FILE; refusing to guess"
  case $expected in
    *[!0-9a-fA-F]*) fail "invalid checksum recorded for $name in $MOLD_SHA256SUMS_FILE" ;;
  esac
  [ "${#expected}" -eq 64 ] ||
    fail "invalid SHA-256 checksum length for $name in $MOLD_SHA256SUMS_FILE"
  printf '%s  %s\n' "$expected" "$archive" | sha256sum --check --status ||
    fail "checksum mismatch for $name; refusing to install"
  note "verified $name against $MOLD_SHA256SUMS_FILE"
}

install_mold() {
  local version=$1 arch name url workdir
  if ! is_linux; then
    note "mold is Linux-only; skipping on $(uname -s)"
    return 0
  fi

  arch=$(mold_arch)
  name="mold-$version-$arch-linux.tar.gz"
  url="$MOLD_RELEASE_BASE_URL/v$version/$name"
  BUILD_TOOLS_WORKDIR=$(mktemp -d)
  workdir=$BUILD_TOOLS_WORKDIR

  note "downloading $url"
  curl --fail --silent --show-error --location \
    --connect-timeout "$CURL_CONNECT_TIMEOUT" \
    --speed-limit "$CURL_MIN_BYTES_PER_SECOND" --speed-time "$CURL_STALL_SECONDS" \
    --output "$workdir/$name" "$url" || fail "failed to download $name"
  verify_mold_archive "$workdir/$name" "$name"

  mkdir -p "$BUILD_TOOLS_PREFIX"
  tar --extract --gzip --strip-components=1 --directory "$BUILD_TOOLS_PREFIX" --file "$workdir/$name" ||
    fail "failed to unpack $name into $BUILD_TOOLS_PREFIX"
  note "installed mold $version into $BUILD_TOOLS_PREFIX"
  note "put $BUILD_TOOLS_PREFIX/bin first on PATH when not using the make targets"
}

install_toolchain() {
  local toolchain=$1 components_text component
  local -a components=()
  command -v rustup >/dev/null 2>&1 || fail 'rustup not found on PATH; install it from https://rustup.rs'
  note "installing toolchain $toolchain"
  rustup toolchain install "$toolchain" --profile minimal ||
    fail "failed to install toolchain $toolchain"
  components_text=$(toolchain_components) || return 1
  if [ -n "$components_text" ]; then
    while IFS= read -r component; do
      components+=("$component")
    done <<< "$components_text"
    note "installing repository components: ${components[*]}"
    rustup component add --toolchain "$toolchain" "${components[@]}" ||
      fail "failed to install components for toolchain $toolchain"
  fi
}

main() {
  local mold_pin toolchain_pin
  [ "$#" -eq 0 ] || fail 'usage: scripts/install-build-tools.sh'
  mold_pin=$(mold_version) || return 1
  toolchain_pin=$(pinned_toolchain) || return 1
  install_mold "$mold_pin"
  install_toolchain "$toolchain_pin"
  note 'ready; verify with: make check-build-tools'
}

if [[ ${BASH_SOURCE[0]} == "$0" ]]; then
  main "$@"
fi

#!/usr/bin/env bash
# Fail before a standard development build if its pinned tools are unavailable.

set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=scripts/build-tools-common.sh
. "$script_dir/build-tools-common.sh"

check_mold() {
  local pinned=$1 resolved installed
  if ! is_linux; then
    note "mold is Linux-only; the native $(uname -s) linker is selected"
    return 0
  fi
  if ! resolved=$(command -v mold 2>/dev/null); then
    note "mold not found on PATH (pinned $pinned)"
    note 'install it with: make install-build-tools'
    return 1
  fi
  if ! installed=$(installed_mold_version) || [ -z "$installed" ]; then
    note "mold at $resolved is unusable"
    note 'reinstall it with: make install-build-tools'
    return 1
  fi
  if [ "$installed" != "$pinned" ]; then
    note "mold $installed at $resolved does not match the pin $pinned"
    note 'run make install-build-tools to match'
    return 1
  fi
  note "mold $installed at $resolved"
}

check_clang() {
  if ! is_linux; then
    return 0
  fi
  local resolved
  if ! resolved=$(command -v clang 2>/dev/null); then
    note 'clang not found on PATH; install the Linux clang runner prerequisite before building'
    return 1
  fi
  if ! clang --version >/dev/null 2>&1; then
    note "clang at $resolved is not executable"
    note 'repair the Linux clang installation before building'
    return 1
  fi
  note "clang available at $resolved (version is supplied by the platform)"
}

check_toolchain() {
  local toolchain=$1
  if ! command -v rustup >/dev/null 2>&1; then
    note 'rustup not found on PATH; install it from https://rustup.rs'
    return 1
  fi
  if ! rustup toolchain list | awk -v expected="$toolchain" \
    '$1 == expected || index($1, expected "-") == 1 { found = 1 } END { exit !found }'; then
    note "toolchain $toolchain is not installed"
    note 'install it with: make install-build-tools'
    return 1
  fi
  note "toolchain $toolchain available"
}

check_toolchain_components() {
  local toolchain=$1 components_text component installed_components status=0
  components_text=$(toolchain_components) || return 1
  [ -n "$components_text" ] || return 0
  if ! installed_components=$(rustup component list --installed --toolchain "$toolchain"); then
    note "could not list installed components for toolchain $toolchain"
    note 'repair the rustup installation, then run make install-build-tools'
    return 1
  fi
  while IFS= read -r component; do
    if ! component_is_installed "$component" "$installed_components"; then
      note "toolchain component $component is not installed for $toolchain"
      status=1
    fi
  done <<< "$components_text"
  if [ "$status" -ne 0 ]; then
    note 'install missing components with: make install-build-tools'
    return 1
  fi
  note 'all rust-toolchain.toml components are installed'
}

main() {
  local status=0 mold_pin toolchain_pin
  [ "$#" -eq 0 ] || fail 'usage: scripts/check-build-tools.sh'
  mold_pin=$(mold_version) || return 1
  toolchain_pin=$(pinned_toolchain) || return 1
  check_mold "$mold_pin" || status=1
  check_clang || status=1
  check_toolchain "$toolchain_pin" || status=1
  check_toolchain_components "$toolchain_pin" || status=1
  [ "$status" -eq 0 ] || note 'capability check failed; see the messages above'
  return "$status"
}

main "$@"

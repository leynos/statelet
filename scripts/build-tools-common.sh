#!/usr/bin/env bash
# Shared helpers for the repository's standard development build tools.

set -euo pipefail

BUILD_TOOLS_SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
BUILD_TOOLS_REPO_ROOT=$(cd -- "$BUILD_TOOLS_SCRIPT_DIR/.." && pwd)

MOLD_VERSION_FILE=${MOLD_VERSION_FILE:-$BUILD_TOOLS_REPO_ROOT/tools/mold/VERSION}
MOLD_SHA256SUMS_FILE=${MOLD_SHA256SUMS_FILE:-$BUILD_TOOLS_REPO_ROOT/tools/mold/SHA256SUMS}
RUST_TOOLCHAIN_FILE=${RUST_TOOLCHAIN_FILE:-$BUILD_TOOLS_REPO_ROOT/rust-toolchain.toml}
BUILD_TOOLS_PREFIX=${BUILD_TOOLS_PREFIX:-$HOME/.local}

note() { printf 'build-tools: %s\n' "$*" >&2; }

fail() {
  note "$*"
  exit 1
}

read_pin() {
  local file=$1 value line_count
  [ -f "$file" ] || fail "missing version pin: $file"
  line_count=$(awk 'END { print NR }' "$file")
  [ "$line_count" -le 1 ] || fail "expected one line in version pin: $file"
  value=$(<"$file")
  value=${value#"${value%%[![:space:]]*}"}
  value=${value%"${value##*[![:space:]]}"}
  [ -n "$value" ] || fail "empty version pin: $file"
  case $value in
    *[[:space:]]*) fail "version pin contains whitespace: $file" ;;
  esac
  printf '%s' "$value"
}

mold_version() { read_pin "$MOLD_VERSION_FILE"; }

pinned_toolchain() {
  local value
  [ -f "$RUST_TOOLCHAIN_FILE" ] || fail "missing toolchain pin: $RUST_TOOLCHAIN_FILE"
  value=$(awk -F '"' '/^[[:space:]]*channel[[:space:]]*=/ { print $2; exit }' "$RUST_TOOLCHAIN_FILE")
  [ -n "$value" ] || fail "no channel found in: $RUST_TOOLCHAIN_FILE"
  case $value in
    *[!A-Za-z0-9._-]*) fail "invalid toolchain channel in: $RUST_TOOLCHAIN_FILE" ;;
  esac
  printf '%s' "$value"
}

toolchain_components() {
  awk '
    function parse_array_line(line, remainder, value) {
      sub(/#.*/, "", line)
      remainder = line
      while (match(remainder, /"[^"]*"/)) {
        value = substr(remainder, RSTART + 1, RLENGTH - 2)
        if (value == "") exit 2
        print value
        remainder = substr(remainder, RSTART + RLENGTH)
      }
      gsub(/[[:space:],\[\]]/, "", remainder)
      if (remainder != "") exit 2
    }
    /^[[:space:]]*\[toolchain\][[:space:]]*$/ { in_toolchain = 1; next }
    /^[[:space:]]*\[[^]]+\][[:space:]]*$/ {
      if (in_components) exit 2
      in_toolchain = 0
    }
    in_toolchain && /^[[:space:]]*components[[:space:]]*=/ {
      if (saw_components++) exit 2
      in_components = 1
      line = $0
      sub(/^[^=]*=[[:space:]]*/, "", line)
      sub(/#.*/, "", line)
      if (line !~ /^[[:space:]]*\[/) exit 2
      parse_array_line(line)
      if (index(line, "]")) {
        in_components = 0
        closed_components = 1
      }
      next
    }
    in_components {
      line = $0
      sub(/#.*/, "", line)
      parse_array_line(line)
      if (index(line, "]")) {
        in_components = 0
        closed_components = 1
      }
    }
    END {
      if (in_components || (saw_components && !closed_components)) exit 2
    }
  ' "$RUST_TOOLCHAIN_FILE" || fail "could not read toolchain components from: $RUST_TOOLCHAIN_FILE"
}

rustup_component_name() {
  case $1 in
    *-preview) printf '%s' "${1%-preview}" ;;
    *) printf '%s' "$1" ;;
  esac
}

component_is_installed() {
  local component=$1 installed_components=$2 rustup_name
  rustup_name=$(rustup_component_name "$component")
  printf '%s\n' "$installed_components" |
    awk -v expected="$rustup_name" \
      '$1 == expected || index($1, expected "-") == 1 { found = 1 } END { exit !found }'
}

is_linux() { [ "$(uname -s)" = 'Linux' ]; }

mold_arch() {
  local machine
  machine=$(uname -m)
  case $machine in
    x86_64 | amd64) printf 'x86_64' ;;
    aarch64 | arm64) printf 'aarch64' ;;
    *) fail "unsupported architecture for the pinned linker release: $machine" ;;
  esac
}

installed_mold_version() {
  local output
  output=$(mold --version 2>/dev/null) || return 1
  printf '%s\n' "$output" | awk 'NR == 1 { print $2; exit }'
}

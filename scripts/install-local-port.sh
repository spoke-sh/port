#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: scripts/install-local-port.sh <source-root> <binary-path> [install-root]

Install a locally built Port binary plus the bundled runtime assets into a
Cargo-style prefix so source-built revisions behave like installed releases.
EOF
}

fail() {
  printf 'error: %s\n' "$1" >&2
  exit 1
}

copy_if_present() {
  local source_path=$1
  local destination_path=$2

  if [[ -f "$source_path" ]]; then
    install -Dm644 "$source_path" "$destination_path"
  fi
}

copy_tree_if_present() {
  local source_path=$1
  local destination_path=$2

  if [[ -d "$source_path" ]]; then
    rm -rf "$destination_path"
    mkdir -p "$(dirname "$destination_path")"
    cp -R "$source_path" "$destination_path"
  fi
}

default_install_root() {
  if [[ -n "${CARGO_HOME:-}" ]]; then
    printf '%s\n' "$CARGO_HOME"
    return 0
  fi

  if [[ -n "${HOME:-}" ]]; then
    printf '%s\n' "$HOME/.cargo"
    return 0
  fi

  fail "set CARGO_HOME or HOME so Port can determine the install root."
}

if [[ $# -lt 2 || $# -gt 3 ]]; then
  usage
  exit 1
fi

source_root=$1
binary_path=$2
install_root=${3:-$(default_install_root)}
bin_dir="$install_root/bin"
share_root="$install_root/share/port"

if [[ ! -d "$source_root" ]]; then
  fail "source root '$source_root' does not exist."
fi

if [[ ! -x "$binary_path" ]]; then
  fail "binary '$binary_path' is not executable."
fi

install -d "$bin_dir" "$share_root"
install -m 0755 "$binary_path" "$bin_dir/port"
copy_if_present "$source_root/README.md" "$share_root/README.md"
copy_if_present "$source_root/RELEASE.md" "$share_root/RELEASE.md"
copy_tree_if_present "$source_root/docs" "$share_root/docs"
copy_tree_if_present "$source_root/examples" "$share_root/examples"
copy_tree_if_present "$source_root/scripts/artifacts" "$share_root/scripts/artifacts"
copy_if_present "$source_root/scripts/install-local-port.sh" "$share_root/scripts/install-local-port.sh"

printf 'installed binary: %s\n' "$bin_dir/port"
printf 'installed share root: %s\n' "$share_root"

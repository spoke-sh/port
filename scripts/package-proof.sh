#!/usr/bin/env bash
set -euo pipefail

umask 022
export TZ=UTC
export COPYFILE_DISABLE=1

usage() {
  cat >&2 <<'EOF'
usage: scripts/package-proof.sh <target-triple> [package-output-dir] [proof-root]

Build the canonical Port package, install it into a clean prefix, and run the
packaged binary through --version, doctor, and guest artifact validation.
EOF
}

fail() {
  printf 'error: %s\n' "$1" >&2
  exit 1
}

require_tool() {
  tool_label=$1
  tool_cmd=$2

  if [[ "$tool_cmd" == */* ]]; then
    if [[ ! -x "$tool_cmd" ]]; then
      fail "missing required packaging tool '$tool_label' at '$tool_cmd'. Install it in the dev environment and rerun \`just package-proof <target>\`."
    fi
    return 0
  fi

  if ! command -v "$tool_cmd" >/dev/null 2>&1; then
    fail "missing required packaging tool '$tool_label'. Install it in the dev environment and rerun \`just package-proof <target>\`."
  fi
}

extract_value() {
  label=$1
  output=$2
  printf '%s\n' "$output" | sed -n "s/^$label: //p" | sed -n '1p'
}

copy_if_present() {
  source_path=$1
  destination_path=$2

  if [[ -f "$source_path" ]]; then
    mkdir -p "$(dirname "$destination_path")"
    cp "$source_path" "$destination_path"
  fi
}

copy_tree_if_present() {
  source_path=$1
  destination_path=$2

  if [[ -d "$source_path" ]]; then
    mkdir -p "$(dirname "$destination_path")"
    cp -R "$source_path" "$destination_path"
  fi
}

if [[ $# -lt 1 || $# -gt 3 ]]; then
  usage
  exit 1
fi

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
target=$1
package_output_dir=${2:-"$repo_root/artifacts/packages"}
proof_root=${3:-"$repo_root/artifacts/package-proof/$target"}
tar_cmd=${PORT_PACKAGE_TAR:-tar}

require_tool tar "$tar_cmd"

rm -rf "$proof_root"
mkdir -p "$proof_root"

package_output=$(bash "$repo_root/scripts/package-port.sh" "$target" "$package_output_dir")
artifact_path=$(extract_value artifact "$package_output")
version=$(extract_value version "$package_output")

if [[ -z "$artifact_path" || -z "$version" ]]; then
  fail "package workflow did not report the artifact path and version."
fi

package_name="port-$version-$target"
extract_root="$proof_root/extracted"
prefix_root="$proof_root/prefix"
package_root="$extract_root/$package_name"
installed_binary="$prefix_root/bin/port"

mkdir -p "$extract_root" "$prefix_root/bin" "$prefix_root/share/port"
"$tar_cmd" -xzf "$artifact_path" -C "$extract_root"

if [[ ! -x "$package_root/bin/port" ]]; then
  fail "extracted package did not contain an executable bin/port at '$package_root/bin/port'."
fi

cp "$package_root/bin/port" "$installed_binary"
chmod 755 "$installed_binary"
copy_if_present "$package_root/README.md" "$prefix_root/share/port/README.md"
copy_if_present "$package_root/RELEASE.md" "$prefix_root/share/port/RELEASE.md"
copy_if_present "$package_root/docs/install.md" "$prefix_root/share/port/docs/install.md"
copy_tree_if_present "$package_root/scripts/artifacts" "$prefix_root/share/port/scripts/artifacts"
copy_if_present "$package_root/PACKAGE_METADATA.txt" "$prefix_root/share/port/PACKAGE_METADATA.txt"
copy_if_present "$package_root/PACKAGE_MANIFEST.txt" "$prefix_root/share/port/PACKAGE_MANIFEST.txt"

version_output=$("$installed_binary" --version)
doctor_output=$("$installed_binary" doctor)
validate_config="$proof_root/validate.toml"
sed \
  -e "s|path = \"artifacts/|path = \"$repo_root/artifacts/|g" \
  -e "s|cache_root = \".port/cache\"|cache_root = \"$repo_root/.port/cache\"|g" \
  -e "s|root = \"artifact-store/|root = \"$repo_root/artifact-store/|g" \
  "$repo_root/examples/port.toml" > "$validate_config"
validate_output=$(
  cd "$proof_root"
  "$installed_binary" \
    --config "$validate_config" \
    artifacts validate \
    --artifact demo-guest \
    --architecture x86-64 2>&1
)

printf '%s\n' "$package_output"
printf 'proof-root: %s\n' "$proof_root"
printf 'prefix: %s\n' "$prefix_root"
printf 'binary: %s\n' "$installed_binary"
printf 'version-output: %s\n' "$version_output"
printf 'doctor-command: %s doctor\n' "$installed_binary"
printf 'doctor-status: ok\n'
printf 'doctor-output:\n%s\n' "$doctor_output"
printf 'validate-command: %s --config %s artifacts validate --artifact demo-guest --architecture x86-64\n' "$installed_binary" "$validate_config"
printf 'validate-status: ok\n'
printf 'validate-output:\n%s\n' "$validate_output"

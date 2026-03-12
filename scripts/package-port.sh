#!/usr/bin/env bash
set -euo pipefail

umask 022
export TZ=UTC
export COPYFILE_DISABLE=1

usage() {
  cat >&2 <<'EOF'
usage: scripts/package-port.sh <target-triple> [output-dir]

Build or stage one canonical Port install package for a supported target.
EOF
}

fail() {
  printf 'error: %s\n' "$1" >&2
  exit 1
}

supported_targets_text() {
  printf '%s\n' 'x86_64-unknown-linux-gnu, x86_64-apple-darwin, aarch64-apple-darwin'
}

is_supported_target() {
  case "$1" in
    x86_64-unknown-linux-gnu|x86_64-apple-darwin|aarch64-apple-darwin)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

require_tool() {
  tool_label=$1
  tool_cmd=$2

  if [[ "$tool_cmd" == */* ]]; then
    if [[ ! -x "$tool_cmd" ]]; then
      fail "missing required packaging tool '$tool_label' at '$tool_cmd'. Install it in the dev environment and rerun \`just package <target>\`."
    fi
    return 0
  fi

  if ! command -v "$tool_cmd" >/dev/null 2>&1; then
    fail "missing required packaging tool '$tool_label'. Install it in the dev environment and rerun \`just package <target>\`."
  fi
}

workspace_version() {
  sed -n '/^\[workspace\.package\]/,/^\[/{s/^version = "\(.*\)"/\1/p}' "$repo_root/Cargo.toml" | sed -n '1p'
}

copy_package_file() {
  source_path=$1
  destination_path=$2
  cp "$source_path" "$destination_path"
}

create_archive() {
  (
    cd "$staging_root"
    if "$tar_cmd" --version 2>/dev/null | grep -q 'GNU tar'; then
      "$tar_cmd" --format=ustar --owner=0 --group=0 --numeric-owner -cf - -T "$archive_members"
    else
      "$tar_cmd" -cf - -T "$archive_members"
    fi
  ) | "$gzip_cmd" -n > "$temporary_archive"
}

if [[ $# -lt 1 || $# -gt 2 ]]; then
  usage
  exit 1
fi

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
target=$1
output_dir=${2:-"$repo_root/artifacts/packages"}
version=${PORT_PACKAGE_VERSION:-$(workspace_version)}
tar_cmd=${PORT_PACKAGE_TAR:-tar}
gzip_cmd=${PORT_PACKAGE_GZIP:-gzip}
cargo_cmd=${PORT_PACKAGE_CARGO:-cargo}
normalized_timestamp=202001010000

if [[ -z "$version" ]]; then
  fail "could not determine the workspace version from Cargo.toml."
fi

if ! is_supported_target "$target"; then
  fail "unsupported package target '$target'. Supported targets: $(supported_targets_text). See docs/install.md for the first-slice support matrix."
fi

require_tool tar "$tar_cmd"
require_tool gzip "$gzip_cmd"

package_name="port-$version-$target"
archive_name="$package_name.tar.gz"
archive_path="$output_dir/$archive_name"
staging_root="$output_dir/.staging"
stage_dir="$staging_root/$package_name"
archive_members="$staging_root/$package_name.members"
temporary_archive="$archive_path.tmp"

binary_source=${PORT_PACKAGE_BIN:-}
if [[ -n "$binary_source" ]]; then
  if [[ ! -x "$binary_source" ]]; then
    fail "PORT_PACKAGE_BIN must point to an executable file."
  fi
else
  require_tool cargo "$cargo_cmd"
  if ! "$cargo_cmd" build -p port-cli --release --target "$target"; then
    fail "cargo build failed for target '$target'. Ensure the Rust toolchain and target support are installed, then rerun \`just package $target\`."
  fi

  cargo_target_dir=${CARGO_TARGET_DIR:-"$repo_root/target"}
  binary_source="$cargo_target_dir/$target/release/port"
  if [[ ! -x "$binary_source" ]]; then
    fail "built binary not found at '$binary_source'. The package workflow only ships the canonical \`port\` binary."
  fi
fi

rm -rf "$stage_dir"
mkdir -p "$stage_dir/bin" "$stage_dir/docs" "$output_dir" "$staging_root"
rm -f "$archive_members" "$archive_path" "$temporary_archive"

copy_package_file "$binary_source" "$stage_dir/bin/port"
chmod 755 "$stage_dir/bin/port"
copy_package_file "$repo_root/README.md" "$stage_dir/README.md"
copy_package_file "$repo_root/RELEASE.md" "$stage_dir/RELEASE.md"
copy_package_file "$repo_root/docs/install.md" "$stage_dir/docs/install.md"
chmod 755 "$stage_dir" "$stage_dir/bin" "$stage_dir/docs"
chmod 644 \
  "$stage_dir/README.md" \
  "$stage_dir/RELEASE.md" \
  "$stage_dir/docs/install.md"

cat > "$stage_dir/PACKAGE_METADATA.txt" <<EOF
package: $package_name
version: $version
target: $target
binary: bin/port
artifact: $archive_name
EOF

cat > "$stage_dir/PACKAGE_MANIFEST.txt" <<'EOF'
bin/port
README.md
RELEASE.md
docs/install.md
PACKAGE_METADATA.txt
PACKAGE_MANIFEST.txt
EOF

cat > "$archive_members" <<EOF
$package_name/bin/port
$package_name/README.md
$package_name/RELEASE.md
$package_name/docs/install.md
$package_name/PACKAGE_METADATA.txt
$package_name/PACKAGE_MANIFEST.txt
EOF

chmod 644 "$stage_dir/PACKAGE_METADATA.txt" "$stage_dir/PACKAGE_MANIFEST.txt"

for path in \
  "$stage_dir" \
  "$stage_dir/bin" \
  "$stage_dir/bin/port" \
  "$stage_dir/docs" \
  "$stage_dir/docs/install.md" \
  "$stage_dir/PACKAGE_METADATA.txt" \
  "$stage_dir/PACKAGE_MANIFEST.txt" \
  "$stage_dir/README.md" \
  "$stage_dir/RELEASE.md"
do
  touch -t "$normalized_timestamp" "$path"
done

create_archive
mv "$temporary_archive" "$archive_path"

printf 'artifact: %s\n' "$archive_path"
printf 'target: %s\n' "$target"
printf 'version: %s\n' "$version"
printf 'included-files:\n'
printf '  - %s\n' \
  'bin/port' \
  'README.md' \
  'RELEASE.md' \
  'docs/install.md' \
  'PACKAGE_METADATA.txt' \
  'PACKAGE_MANIFEST.txt'

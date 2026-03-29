#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <guest-image-path>" >&2
  exit 2
fi

guest_image_path="$1"
if [[ ! -f "$guest_image_path" ]]; then
  echo "missing guest image artifact: $guest_image_path" >&2
  exit 1
fi

case "$guest_image_path" in
  */x86_64/firecracker/standard/*)
    expected_architecture="x86_64"
    expected_protection_mode="standard"
    ;;
  */x86_64/firecracker/pvm/*)
    expected_architecture="x86_64"
    expected_protection_mode="pvm"
    ;;
  */aarch64/firecracker/standard/*)
    expected_architecture="aarch64"
    expected_protection_mode="standard"
    ;;
  */aarch64/firecracker/pvm/*)
    echo "aarch64/firecracker/pvm remains research-only and has no guest-image validation pipeline yet" >&2
    exit 1
    ;;
  *)
    echo "unsupported artifact selector for demo guest-image validation: $guest_image_path" >&2
    exit 1
    ;;
esac

for tool in debugfs e2fsck; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "missing required tool for guest image validation: $tool" >&2
    exit 1
  fi
done

require_debugfs_path() {
  local path="$1"
  local output
  output="$(debugfs -R "stat $path" "$guest_image_path" 2>&1 || true)"
  if ! printf '%s\n' "$output" | grep -q 'Inode:'; then
    echo "guest image is missing required path: $path" >&2
    exit 1
  fi
}

e2fsck -fn "$guest_image_path" >/dev/null
require_debugfs_path /init
require_debugfs_path /bin/busybox
require_debugfs_path /bin/dirname
require_debugfs_path /bin/install
require_debugfs_path /usr/bin/port-guest-agent
if [[ "$(debugfs -R 'cat /etc/port-guest-architecture' "$guest_image_path" 2>/dev/null | tr -d '\r\n')" != "$expected_architecture" ]]; then
  echo "guest image architecture marker does not match $expected_architecture" >&2
  exit 1
fi
if [[ "$(debugfs -R 'cat /etc/port-protection-mode' "$guest_image_path" 2>/dev/null | tr -d '\r\n')" != "$expected_protection_mode" ]]; then
  echo "guest image protection marker does not match $expected_protection_mode" >&2
  exit 1
fi
if ! debugfs -R 'cat /init' "$guest_image_path" 2>/dev/null | grep -q 'port-guest-agent'; then
  echo "guest image init does not launch port-guest-agent" >&2
  exit 1
fi
if ! debugfs -R 'cat /init' "$guest_image_path" 2>/dev/null | grep -q '/etc/port-protection-mode'; then
  echo "guest image init does not load the protection-mode marker" >&2
  exit 1
fi

guest_image_size="$(stat -c '%s' "$guest_image_path")"
printf 'validated guest image: %s\n' "$guest_image_path"
printf 'guest image architecture: %s\n' "$expected_architecture"
printf 'guest image protection mode: %s\n' "$expected_protection_mode"
printf 'guest image bytes: %s\n' "$guest_image_size"

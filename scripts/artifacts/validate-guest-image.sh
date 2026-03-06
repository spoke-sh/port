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

for tool in debugfs e2fsck; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "missing required tool for guest image validation: $tool" >&2
    exit 1
  fi
done

e2fsck -fn "$guest_image_path" >/dev/null
debugfs -R 'stat /init' "$guest_image_path" >/dev/null 2>&1
debugfs -R 'stat /bin/busybox' "$guest_image_path" >/dev/null 2>&1
debugfs -R 'stat /usr/bin/port-guest-agent' "$guest_image_path" >/dev/null 2>&1
if ! debugfs -R 'cat /init' "$guest_image_path" 2>/dev/null | grep -q 'port-guest-agent'; then
  echo "guest image init does not launch port-guest-agent" >&2
  exit 1
fi

guest_image_size="$(stat -c '%s' "$guest_image_path")"
printf 'validated guest image: %s\n' "$guest_image_path"
printf 'guest image bytes: %s\n' "$guest_image_size"

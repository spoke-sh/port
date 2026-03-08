#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <output-path>" >&2
  exit 2
fi

output_path="$1"
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

case "$output_path" in
  */x86_64/firecracker/standard/*)
    guest_architecture="x86_64"
    protection_mode="standard"
    ;;
  */x86_64/firecracker/pvm/*)
    guest_architecture="x86_64"
    protection_mode="pvm"
    ;;
  */aarch64/firecracker/standard/*)
    guest_architecture="aarch64"
    protection_mode="standard"
    ;;
  */aarch64/firecracker/pvm/*)
    echo "aarch64/firecracker/pvm remains research-only and has no guest-image build pipeline yet" >&2
    exit 1
    ;;
  *)
    echo "unsupported artifact selector for demo guest-image pipeline: $output_path" >&2
    exit 1
    ;;
esac

for tool in busybox cargo e2fsck ldd mkfs.ext4; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "missing required tool for guest image build: $tool" >&2
    exit 1
  fi
done

staging_dir="$(mktemp -d)"
image_path="$(mktemp)"
trap 'rm -rf "$staging_dir"; rm -f "$image_path"' EXIT

copy_binary_with_libs() {
  local source="$1"
  local destination="$2"

  install -D "$source" "$staging_dir/$destination"
  while IFS= read -r library; do
    [[ -n "$library" ]] || continue
    install -D "$library" "$staging_dir/$library"
  done < <(
    ldd "$source" | awk '
      {
        for (i = 1; i <= NF; i++) {
          if ($i ~ /^\//) {
            print $i
          }
        }
      }
    ' | sort -u
  )
}

mkdir -p \
  "$staging_dir/bin" \
  "$staging_dir/dev" \
  "$staging_dir/etc" \
  "$staging_dir/proc" \
  "$staging_dir/run/port" \
  "$staging_dir/sys" \
  "$staging_dir/tmp" \
  "$staging_dir/usr/bin" \
  "$staging_dir/var/log"

cat >"$staging_dir/etc/group" <<'EOF'
root:x:0:
EOF

cat >"$staging_dir/etc/passwd" <<'EOF'
root:x:0:0:root:/root:/bin/sh
EOF

printf '%s\n' "$guest_architecture" >"$staging_dir/etc/port-guest-architecture"
printf '%s\n' "$protection_mode" >"$staging_dir/etc/port-protection-mode"

cargo build -p port-guest-agent --release

copy_binary_with_libs \
  "$(readlink -f "$(command -v busybox)")" \
  "bin/busybox"
copy_binary_with_libs \
  "${CARGO_TARGET_DIR:-$repo_root/target}/release/port-guest-agent" \
  "usr/bin/port-guest-agent"

for applet in cat chmod echo ln ls mkdir mount mknod setsid sh sleep uname; do
  ln -sf busybox "$staging_dir/bin/$applet"
done

cat >"$staging_dir/init" <<'EOF'
#!/bin/sh
set -eu

mount_if_needed() {
  mount -t "$1" "$2" "$3" 2>/dev/null || true
}

mount_if_needed devtmpfs devtmpfs /dev
mkdir -p /dev/pts
mount_if_needed devpts devpts /dev/pts
mount_if_needed proc proc /proc
mount_if_needed sysfs sysfs /sys
mkdir -p /run/port /tmp /var/log

guest_control_port=7000
protection_mode="$(cat /etc/port-protection-mode 2>/dev/null || echo unknown)"
for token in $(cat /proc/cmdline); do
  case "$token" in
    port.guest_control_port=*)
      guest_control_port="${token#port.guest_control_port=}"
      ;;
  esac
done

echo "port guest image booted" >/dev/console
echo "port guest image booted" >>/var/log/port-agent.log
echo "port protection mode: ${protection_mode}" >/dev/console
echo "port protection mode: ${protection_mode}" >>/var/log/port-agent.log
/usr/bin/port-guest-agent --socket /run/port/guest-agent.sock --vsock-port "$guest_control_port" --root / >>/var/log/port-agent.log 2>&1 &
echo "port-guest-agent launched on vsock port $guest_control_port" >/dev/console
echo "port-guest-agent launched on vsock port $guest_control_port" >>/var/log/port-agent.log

while true; do
  sleep 3600
done
EOF
chmod 0755 "$staging_dir/init"

mkdir -p "$(dirname "$output_path")"
truncate -s 256M "$image_path"
mkfs.ext4 -q -F -L port-demo-rootfs -d "$staging_dir" "$image_path"
install -m 0644 "$image_path" "$output_path"
e2fsck -fn "$output_path" >/dev/null

printf 'guest image artifact: %s\n' "$output_path"
printf 'guest image architecture: %s\n' "$guest_architecture"
printf 'guest image protection mode: %s\n' "$protection_mode"
printf 'guest image contains: /init, /bin/busybox, /usr/bin/port-guest-agent\n'

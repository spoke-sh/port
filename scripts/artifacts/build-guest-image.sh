#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <output-path>" >&2
  exit 2
fi

output_path="$1"
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

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

mount -t devtmpfs devtmpfs /dev
mkdir -p /dev/pts
mount -t devpts devpts /dev/pts
mount -t proc proc /proc
mount -t sysfs sysfs /sys
mkdir -p /run/port /tmp /var/log

echo "port guest image booted" >/dev/console
echo "port guest image booted" >>/var/log/port-agent.log
/usr/bin/port-guest-agent --socket /run/port/guest-agent.sock --root / >>/var/log/port-agent.log 2>&1 &
echo "port-guest-agent launched" >/dev/console
echo "port-guest-agent launched" >>/var/log/port-agent.log

exec /bin/sh </dev/console >/dev/console 2>&1
EOF
chmod 0755 "$staging_dir/init"

mkdir -p "$(dirname "$output_path")"
truncate -s 256M "$image_path"
mkfs.ext4 -q -F -L port-demo-rootfs -d "$staging_dir" "$image_path"
install -m 0644 "$image_path" "$output_path"
e2fsck -fn "$output_path" >/dev/null

printf 'guest image artifact: %s\n' "$output_path"
printf 'guest image contains: /init, /bin/busybox, /usr/bin/port-guest-agent\n'

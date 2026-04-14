#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <output-path>" >&2
  exit 2
fi

output_path="$1"
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"
busybox_bin="${PORT_BUSYBOX_BIN:-$(command -v busybox || true)}"

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

if [[ -z "$busybox_bin" ]]; then
  echo "missing required tool for guest image build: busybox (set PORT_BUSYBOX_BIN or install busybox on the host)" >&2
  exit 1
fi

for tool in cargo e2fsck ldd mkfs.ext4 nix nix-store; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "missing required tool for guest image build: $tool" >&2
    exit 1
  fi
done

cp_bin="${PORT_CP_BIN:-}"
if [[ -z "$cp_bin" ]]; then
  cp_store="$(nix build --option eval-cache false --no-link --print-out-paths nixpkgs#coreutils)"
  cp_bin="${cp_store}/bin/cp"
fi
if [[ ! -x "$cp_bin" ]]; then
  echo "missing required tool for guest image build: GNU cp (set PORT_CP_BIN or install coreutils)" >&2
  exit 1
fi

build_firecracker_initrd=0
copy_kernel_modules_into_guest=0
stage_preloaded_k3s_images=0
case "$output_path" in
  */x86_64/firecracker/standard/*)
    build_firecracker_initrd=1
    copy_kernel_modules_into_guest=1
    stage_preloaded_k3s_images=1
    ;;
  */x86_64/firecracker/pvm/*)
    build_firecracker_initrd=1
    stage_preloaded_k3s_images=1
    ;;
esac

if [[ "$build_firecracker_initrd" -eq 1 ]]; then
  for tool in cpio gzip; do
    if ! command -v "$tool" >/dev/null 2>&1; then
      echo "missing required tool for guest image build: $tool" >&2
      exit 1
    fi
  done
fi

if [[ "$stage_preloaded_k3s_images" -eq 1 ]]; then
  if ! command -v skopeo >/dev/null 2>&1; then
    echo "missing required tool for guest image build: skopeo" >&2
    exit 1
  fi
fi

case "$guest_architecture" in
  x86_64)
    guest_nix_system="x86_64-linux"
    ;;
  aarch64)
    guest_nix_system="aarch64-linux"
    ;;
  *)
    echo "unsupported guest architecture for demo guest-image pipeline: $guest_architecture" >&2
    exit 1
    ;;
esac

staging_dir="$(mktemp -d)"
initrd_dir="$(mktemp -d)"
image_path="$(mktemp)"
initrd_path="$(mktemp)"
trap 'chmod -R u+w "$staging_dir" "$initrd_dir" 2>/dev/null || true; rm -rf "$staging_dir" "$initrd_dir"; rm -f "$image_path" "$initrd_path"' EXIT

copy_binary_with_libs_into() {
  local root="$1"
  local source="$2"
  local destination="$3"

  install -D "$source" "$root/$destination"
  while IFS= read -r library; do
    [[ -n "$library" ]] || continue
    install -D "$library" "$root/$library"
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

copy_binary_with_libs() {
  copy_binary_with_libs_into "$staging_dir" "$1" "$2"
}

copy_store_closure() {
  local attr="$1"
  local output_path
  local primary_output=""

  while IFS= read -r output_path; do
    [[ -n "$output_path" ]] || continue
    if [[ -z "$primary_output" ]]; then
      primary_output="$output_path"
    fi
    while IFS= read -r closure_path; do
      [[ -n "$closure_path" ]] || continue
      if [[ -e "$staging_dir/$closure_path" ]]; then
        continue
      fi
      mkdir -p "$staging_dir/$(dirname "$closure_path")"
      "$cp_bin" -a --no-preserve=ownership "$closure_path" "$staging_dir/$(dirname "$closure_path")/"
      chmod -R u+w "$staging_dir/$closure_path"
    done < <(nix-store -qR "$output_path")
  done < <(nix build --option eval-cache false --no-link --print-out-paths "$attr")

  if [[ -z "$primary_output" ]]; then
    echo "failed to resolve any store outputs for $attr" >&2
    exit 1
  fi

  printf '%s\n' "$primary_output"
}

sanitize_image_ref() {
  printf '%s' "$1" | tr '/:@' '___'
}

stage_preloaded_images() {
  local manifest_path="$1"
  local cache_root="$2"
  local image_root="$staging_dir/var/lib/rancher/k3s/agent/images"
  local recorded=0

  mkdir -p "$cache_root" "$image_root"
  : >"$staging_dir/etc/port-preloaded-images.txt"
  while IFS= read -r image || [[ -n "$image" ]]; do
    [[ -n "$image" ]] || continue
    [[ "$image" == \#* ]] && continue

    local archive_name
    archive_name="$(sanitize_image_ref "$image").tar"
    local cached_archive="$cache_root/$archive_name"
    if [[ ! -f "$cached_archive" ]]; then
      skopeo copy --insecure-policy --retry-times 3 \
        "docker://$image" \
        "docker-archive:${cached_archive}:${image}"
    fi

    install -Dm0644 "$cached_archive" "$image_root/$archive_name"
    printf '%s\n' "$image" >>"$staging_dir/etc/port-preloaded-images.txt"
    recorded=1
  done <"$manifest_path"

  if [[ "$recorded" -ne 1 ]]; then
    echo "preloaded image manifest did not contain any images: $manifest_path" >&2
    exit 1
  fi
}

mkdir -p \
  "$staging_dir/bin" \
  "$staging_dir/dev" \
  "$staging_dir/etc" \
  "$staging_dir/etc/pki/tls/certs" \
  "$staging_dir/etc/rancher/k3s" \
  "$staging_dir/etc/ssl/certs" \
  "$staging_dir/nix/store" \
  "$staging_dir/opt/credential-provider/bin" \
  "$staging_dir/opt/cni/bin" \
  "$staging_dir/proc" \
  "$staging_dir/run/port" \
  "$staging_dir/sys" \
  "$staging_dir/sys/fs/cgroup" \
  "$staging_dir/tmp" \
  "$staging_dir/usr/bin" \
  "$staging_dir/var/lib/rancher/k3s/agent/images" \
  "$staging_dir/var/lib/cni" \
  "$staging_dir/var/lib/kubelet" \
  "$staging_dir/var/lib/rancher/k3s" \
  "$staging_dir/var/log"

cat >"$staging_dir/etc/group" <<'EOF'
root:x:0:
EOF

cat >"$staging_dir/etc/passwd" <<'EOF'
root:x:0:0:root:/root:/bin/sh
EOF

printf '%s\n' "$guest_architecture" >"$staging_dir/etc/port-guest-architecture"
printf '%s\n' "$protection_mode" >"$staging_dir/etc/port-protection-mode"

k3s_attr="nixpkgs#legacyPackages.${guest_nix_system}.k3s"
k3s_store="$(copy_store_closure "$k3s_attr")"
k3s_version="$(nix eval --option eval-cache false --raw "${k3s_attr}.version")"
printf '%s\n' "$k3s_version" >"$staging_dir/etc/port-k3s-version"
printf '%s\n' "$k3s_store" >"$staging_dir/etc/port-k3s-store-path"
kmod_attr="nixpkgs#legacyPackages.${guest_nix_system}.kmod.out"
kmod_store="$(copy_store_closure "$kmod_attr")"
ln -sf "${kmod_store}/bin/modprobe" "$staging_dir/usr/bin/modprobe"
ln -sf "${kmod_store}/bin/lsmod" "$staging_dir/usr/bin/lsmod"
iptables_attr="nixpkgs#legacyPackages.${guest_nix_system}.iptables.out"
iptables_store="$(copy_store_closure "$iptables_attr")"
nftables_attr="nixpkgs#legacyPackages.${guest_nix_system}.nftables.out"
nftables_store="$(copy_store_closure "$nftables_attr")"
awscli_attr="nixpkgs#legacyPackages.${guest_nix_system}.awscli2"
awscli_store="$(copy_store_closure "$awscli_attr")"
cacert_attr="nixpkgs#legacyPackages.${guest_nix_system}.cacert.out"
cacert_store="$(copy_store_closure "$cacert_attr")"
for tool in \
  iptables \
  iptables-save \
  iptables-restore \
  ip6tables \
  ip6tables-save \
  ip6tables-restore
do
  ln -sf "${iptables_store}/bin/xtables-nft-multi" "$staging_dir/usr/bin/$tool"
done
ln -sf "${iptables_store}/bin/xtables-nft-multi" "$staging_dir/usr/bin/xtables-nft-multi"
ln -sf "${iptables_store}/bin/xtables-legacy-multi" "$staging_dir/usr/bin/xtables-legacy-multi"
ln -sf "${nftables_store}/bin/nft" "$staging_dir/usr/bin/nft"
ln -sf "${awscli_store}/bin/aws" "$staging_dir/usr/bin/aws"
ln -sf "${cacert_store}/etc/ssl/certs/ca-bundle.crt" "$staging_dir/etc/ssl/certs/ca-certificates.crt"
ln -sf /etc/ssl/certs/ca-certificates.crt "$staging_dir/etc/pki/tls/certs/ca-bundle.crt"

cat >"$staging_dir/etc/rancher/k3s/ecr-credential-provider.yaml" <<'EOF'
apiVersion: kubelet.config.k8s.io/v1
kind: CredentialProviderConfig
providers:
  - name: ecr-credential-provider
    apiVersion: credentialprovider.kubelet.k8s.io/v1
    matchImages:
      - "*.dkr.ecr.*.amazonaws.com"
      - "*.dkr.ecr.*.amazonaws.com.cn"
    defaultCacheDuration: 11h
EOF

cat >"$staging_dir/opt/credential-provider/bin/ecr-credential-provider" <<'EOF'
#!/bin/sh
set -eu

request_file="$(/bin/mktemp)"
trap '/bin/rm -f "$request_file"' EXIT
/bin/cat >"$request_file"

api_version="$(/bin/sed -n 's/.*"apiVersion"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$request_file" | /bin/head -n 1)"
if [ -z "$api_version" ]; then
  api_version="credentialprovider.kubelet.k8s.io/v1"
fi
image="$(/bin/sed -n 's/.*"image"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$request_file" | /bin/head -n 1)"
registry_host="${image%%/*}"

if [ -z "$image" ] || [ -z "$registry_host" ]; then
  echo "ecr-credential-provider: missing image in kubelet request" >&2
  exit 1
fi

case "$registry_host" in
  *.dkr.ecr.*.amazonaws.com|*.dkr.ecr.*.amazonaws.com.cn)
    ;;
  *)
    /bin/printf '{"apiVersion":"%s","kind":"CredentialProviderResponse","cacheKeyType":"Registry","auth":null}\n' "$api_version"
    exit 0
    ;;
esac

region="$(/bin/printf '%s' "$registry_host" | /bin/tr '.' '\n' | /bin/sed -n '4p')"
if [ -z "$region" ]; then
  echo "ecr-credential-provider: could not derive region from registry host '$registry_host'" >&2
  exit 1
fi

password="$(/usr/bin/aws ecr get-login-password --region "$region")"

/bin/printf '{"apiVersion":"%s","kind":"CredentialProviderResponse","cacheKeyType":"Registry","cacheDuration":"11h","auth":{"%s":{"username":"AWS","password":"%s"}}}\n' \
  "$api_version" \
  "$registry_host" \
  "$password"
EOF
chmod 0755 "$staging_dir/opt/credential-provider/bin/ecr-credential-provider"

if [[ "$copy_kernel_modules_into_guest" -eq 1 ]]; then
  kernel_modules_store="$(nix build --option eval-cache false --no-link --print-out-paths nixpkgs#linuxPackages_latest.kernel.modules)"
  if [[ ! -d "${kernel_modules_store}/lib/modules" ]]; then
    echo "missing kernel modules in ${kernel_modules_store}/lib/modules" >&2
    exit 1
  fi
  mkdir -p "$staging_dir/lib"
  "$cp_bin" -a --no-preserve=ownership "${kernel_modules_store}/lib/modules" "$staging_dir/lib/"
  chmod -R u+w "$staging_dir/lib/modules"
  kernel_release="$(find "${kernel_modules_store}/lib/modules" -mindepth 1 -maxdepth 1 -type d | head -n 1 | xargs basename)"
  if [[ -z "$kernel_release" ]]; then
    echo "failed to determine kernel release from ${kernel_modules_store}/lib/modules" >&2
    exit 1
  fi
  printf '%s\n' "$kernel_release" >"$staging_dir/etc/port-kernel-modules-release"
fi

if [[ "$stage_preloaded_k3s_images" -eq 1 ]]; then
  stage_preloaded_images \
    "$repo_root/examples/bootstrap/demo-k3s/preloaded-images.txt" \
    "${PORT_PRELOADED_IMAGE_CACHE_ROOT:-${XDG_CACHE_HOME:-$HOME/.cache}/port/preloaded-images}"
fi

if [[ "$build_firecracker_initrd" -eq 1 ]]; then
  mkdir -p "$initrd_dir/bin" "$initrd_dir/dev" "$initrd_dir/etc" "$initrd_dir/lib" "$initrd_dir/newroot" "$initrd_dir/proc" "$initrd_dir/sys"
  copy_binary_with_libs_into \
    "$initrd_dir" \
    "$(readlink -f "$busybox_bin")" \
    "bin/busybox"
  for applet in cat echo ip mkdir mount sh sleep switch_root; do
    ln -sf busybox "$initrd_dir/bin/$applet"
  done
  if [[ "$copy_kernel_modules_into_guest" -eq 1 ]]; then
    copy_binary_with_libs_into \
      "$initrd_dir" \
      "${kmod_store}/bin/modprobe" \
      "bin/modprobe"
    "$cp_bin" -a --no-preserve=ownership "${kernel_modules_store}/lib/modules" "$initrd_dir/lib/"
    chmod -R u+w "$initrd_dir/lib/modules"
    printf '%s\n' "$kernel_release" >"$initrd_dir/etc/port-kernel-modules-release"
  fi
  cat >"$initrd_dir/init" <<'EOF'
#!/bin/sh
set -eu

PATH=/bin
mount -t devtmpfs devtmpfs /dev 2>/dev/null || true
mount -t proc proc /proc 2>/dev/null || true
mount -t sysfs sysfs /sys 2>/dev/null || true

if [ -x /bin/modprobe ] && [ -f /etc/port-kernel-modules-release ]; then
  kernel_release="$(cat /etc/port-kernel-modules-release)"
  /bin/modprobe -d / -S "$kernel_release" virtio_mmio || true
  /bin/modprobe -d / -S "$kernel_release" virtio_blk || true
  /bin/modprobe -d / -S "$kernel_release" ext4 || true
  /bin/modprobe -d / -S "$kernel_release" overlay || true
  /bin/modprobe -d / -S "$kernel_release" vsock || true
  /bin/modprobe -d / -S "$kernel_release" vmw_vsock_virtio_transport || true
  /bin/modprobe -d / -S "$kernel_release" virtio_net || true
fi

overlay_enabled=0
overlay_device="/dev/vdb"
for token in $(cat /proc/cmdline); do
  case "$token" in
    port.rootfs_overlay=1)
      overlay_enabled=1
      ;;
    port.rootfs_overlay_device=*)
      overlay_device="${token#port.rootfs_overlay_device=}"
      ;;
  esac
done

mkdir -p /lower /overlay /newroot

for _ in 1 2 3 4 5; do
  if [ "$overlay_enabled" -eq 1 ]; then
    if mount -t ext4 -o ro /dev/vda /lower 2>/dev/null; then
      if mount -t ext4 -o rw "$overlay_device" /overlay 2>/dev/null; then
        mkdir -p /overlay/upper /overlay/work
        if mount -t overlay overlay -o lowerdir=/lower,upperdir=/overlay/upper,workdir=/overlay/work /newroot 2>/dev/null; then
          exec switch_root /newroot /init
        fi
        umount /overlay 2>/dev/null || true
      fi
      umount /lower 2>/dev/null || true
    fi
  elif mount -t ext4 -o rw /dev/vda /newroot 2>/dev/null; then
    exec switch_root /newroot /init
  fi
  sleep 1
done

echo "failed to mount rootfs from initrd" >/dev/console
exec sh
EOF
  chmod 0755 "$initrd_dir/init"
  (
    cd "$initrd_dir"
    find . -print0 | cpio --null -o -H newc | gzip -n >"$initrd_path"
  )
  initrd_output_path="$(dirname "$output_path")/initrd.cpio.gz"
fi

cargo build -p port-guest-agent --release

copy_binary_with_libs \
  "$(readlink -f "$busybox_bin")" \
  "bin/busybox"
copy_binary_with_libs \
  "${CARGO_TARGET_DIR:-$repo_root/target}/release/port-guest-agent" \
  "usr/bin/port-guest-agent"

for applet in cat chmod cp dirname echo env grep head install ip kill ln ls mkdir mktemp mount mknod mv ps printf pwd readlink rm sed setsid sh sleep stat sync tail touch tr uname; do
  ln -sf busybox "$staging_dir/bin/$applet"
done

cat >"$staging_dir/usr/bin/k3s" <<EOF
#!/bin/sh
set -eu

export SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
export SSL_CERT_DIR=/etc/ssl/certs

exec '${k3s_store}/bin/k3s' "\$@"
EOF
chmod 0755 "$staging_dir/usr/bin/k3s"
ln -sf k3s "$staging_dir/usr/bin/kubectl"
ln -sf k3s "$staging_dir/usr/bin/crictl"
ln -sf k3s "$staging_dir/usr/bin/ctr"

cat >"$staging_dir/etc/hosts" <<'EOF'
127.0.0.1 localhost
::1 localhost
EOF

cat >"$staging_dir/init" <<'EOF'
#!/bin/sh
set -eu
PATH=/bin:/usr/bin

mount_if_needed() {
  mount -t "$1" "$2" "$3" 2>/dev/null || true
}

mount_if_needed devtmpfs devtmpfs /dev
mkdir -p /dev/pts
mount_if_needed devpts devpts /dev/pts
mount_if_needed proc proc /proc
mount_if_needed sysfs sysfs /sys
mkdir -p /run/port /tmp /var/log /sys/fs/cgroup
mount_if_needed cgroup2 cgroup2 /sys/fs/cgroup
mkdir -p /etc/rancher/k3s /opt/cni/bin /var/lib/cni /var/lib/kubelet /var/lib/rancher/k3s /var/lib/rancher/k3s/agent/images
/bin/ip link set lo up 2>/dev/null || true
/bin/ip addr add 127.0.0.1/8 dev lo 2>/dev/null || true
/bin/ip -6 addr add ::1/128 dev lo 2>/dev/null || true

guest_control_port=7000
protection_mode="$(cat /etc/port-protection-mode 2>/dev/null || echo unknown)"
net_ip=""
net_gateway=""
net_prefix_len="24"
net_dns=""
guest_hostname=""
for token in $(cat /proc/cmdline); do
  case "$token" in
    port.guest_control_port=*)
      guest_control_port="${token#port.guest_control_port=}"
      ;;
    port.hostname=*)
      guest_hostname="${token#port.hostname=}"
      ;;
    port.net_ip=*)
      net_ip="${token#port.net_ip=}"
      ;;
    port.net_gateway=*)
      net_gateway="${token#port.net_gateway=}"
      ;;
    port.net_prefix_len=*)
      net_prefix_len="${token#port.net_prefix_len=}"
      ;;
    port.net_dns=*)
      net_dns="${token#port.net_dns=}"
      ;;
  esac
done

echo "port guest image booted" >/dev/console
echo "port guest image booted" >>/var/log/port-agent.log
echo "port protection mode: ${protection_mode}" >/dev/console
echo "port protection mode: ${protection_mode}" >>/var/log/port-agent.log
if [ -f /etc/port-k3s-version ]; then
  k3s_version="$(cat /etc/port-k3s-version)"
  echo "port k3s version: ${k3s_version}" >/dev/console
  echo "port k3s version: ${k3s_version}" >>/var/log/port-agent.log
fi

if [ -n "$guest_hostname" ]; then
  printf '%s\n' "$guest_hostname" >/proc/sys/kernel/hostname 2>/dev/null || true
  printf '%s\n' "$guest_hostname" >/etc/hostname
  if ! /bin/busybox grep -q "127.0.1.1 ${guest_hostname}" /etc/hosts 2>/dev/null; then
    printf '127.0.1.1 %s\n' "$guest_hostname" >>/etc/hosts
  fi
  echo "port guest hostname: ${guest_hostname}" >/dev/console
  echo "port guest hostname: ${guest_hostname}" >>/var/log/port-agent.log
fi

if [ -n "$net_ip" ] && [ -n "$net_gateway" ]; then
  for _ in 1 2 3 4 5; do
    if /bin/ip link show eth0 >/dev/null 2>&1; then
      break
    fi
    sleep 1
  done

  if /bin/ip link show eth0 >/dev/null 2>&1; then
    /bin/ip addr add "${net_ip}/${net_prefix_len}" dev eth0
    /bin/ip link set eth0 up
    /bin/ip route add default via "$net_gateway" dev eth0
    echo "port guest network: ${net_ip}/${net_prefix_len} via ${net_gateway}" >/dev/console
    echo "port guest network: ${net_ip}/${net_prefix_len} via ${net_gateway}" >>/var/log/port-agent.log
  else
    echo "port guest network: eth0 not found, skipping network setup" >/dev/console
    echo "port guest network: eth0 not found, skipping network setup" >>/var/log/port-agent.log
  fi

  if [ -n "$net_dns" ]; then
    : >/etc/resolv.conf
    old_IFS="$IFS"
    IFS=','
    for server in $net_dns; do
      printf 'nameserver %s\n' "$server" >>/etc/resolv.conf
    done
    IFS="$old_IFS"
    dns_summary="$(/bin/tr '\n' ' ' </etc/resolv.conf 2>/dev/null || cat /etc/resolv.conf)"
    echo "port guest dns: configured ${dns_summary}" >/dev/console
    echo "port guest dns: configured ${dns_summary}" >>/var/log/port-agent.log
  fi
fi

/usr/bin/port-guest-agent --socket /run/port/guest-agent.sock --vsock-port "$guest_control_port" --root / >>/var/log/port-agent.log 2>&1 &
echo "port-guest-agent launched on vsock port $guest_control_port" >/dev/console
echo "port-guest-agent launched on vsock port $guest_control_port" >>/var/log/port-agent.log

while true; do
  sleep 3600
done
EOF
chmod 0755 "$staging_dir/init"

mkdir -p "$(dirname "$output_path")"
truncate -s "${PORT_DEMO_GUEST_IMAGE_SIZE:-12G}" "$image_path"
mkfs.ext4 -q -F -L port-demo-rootfs -d "$staging_dir" "$image_path"
install -m 0644 "$image_path" "$output_path"
if [[ -n "${initrd_output_path:-}" ]]; then
  install -m 0644 "$initrd_path" "$initrd_output_path"
fi
e2fsck -fn "$output_path" >/dev/null

printf 'guest image artifact: %s\n' "$output_path"
printf 'guest image architecture: %s\n' "$guest_architecture"
printf 'guest image protection mode: %s\n' "$protection_mode"
printf 'guest image K3s version: %s\n' "$k3s_version"
if [[ -n "${initrd_output_path:-}" ]]; then
  printf 'guest image initrd: %s\n' "$initrd_output_path"
fi
printf 'guest image contains: /init, /bin/busybox, /usr/bin/port-guest-agent, /usr/bin/k3s\n'

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
require_debugfs_path /bin/mktemp
require_debugfs_path /bin/printf
require_debugfs_path /bin/tr
require_debugfs_path /usr/bin/port-guest-agent
require_debugfs_path /usr/bin/k3s
require_debugfs_path /usr/bin/kubectl
require_debugfs_path /usr/bin/iptables
require_debugfs_path /usr/bin/iptables-save
require_debugfs_path /usr/bin/ip6tables
require_debugfs_path /usr/bin/nft
require_debugfs_path /usr/bin/aws
require_debugfs_path /opt/credential-provider/bin/ecr-credential-provider
require_debugfs_path /etc/rancher/k3s/ecr-credential-provider.yaml
require_debugfs_path /etc/ssl/certs/ca-certificates.crt
require_debugfs_path /etc/pki/tls/certs/ca-bundle.crt
require_debugfs_path /etc/port-k3s-version
require_debugfs_path /etc/port-k3s-store-path
require_debugfs_path /etc/hosts
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
if ! debugfs -R 'cat /init' "$guest_image_path" 2>/dev/null | grep -q 'port.hostname='; then
  echo "guest image init does not parse the guest hostname kernel arg" >&2
  exit 1
fi
if ! debugfs -R 'cat /init' "$guest_image_path" 2>/dev/null | grep -q 'cgroup2'; then
  echo "guest image init does not mount cgroup2 for K3s" >&2
  exit 1
fi
if ! debugfs -R 'cat /init' "$guest_image_path" 2>/dev/null | grep -q 'ip link set lo up'; then
  echo "guest image init does not bring loopback up before guest services" >&2
  exit 1
fi
if ! debugfs -R 'cat /init' "$guest_image_path" 2>/dev/null | grep -q '127.0.0.1/8'; then
  echo "guest image init does not assign 127.0.0.1/8 to loopback" >&2
  exit 1
fi
if ! debugfs -R 'cat /usr/bin/k3s' "$guest_image_path" 2>/dev/null | grep -q '/bin/k3s'; then
  echo "guest image k3s wrapper does not point at the packaged K3s runtime" >&2
  exit 1
fi
if ! debugfs -R 'cat /usr/bin/k3s' "$guest_image_path" 2>/dev/null | grep -q 'SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt'; then
  echo "guest image k3s wrapper does not export SSL_CERT_FILE for outbound registry pulls" >&2
  exit 1
fi
if ! debugfs -R 'cat /opt/credential-provider/bin/ecr-credential-provider' "$guest_image_path" 2>/dev/null | grep -q 'aws ecr get-login-password'; then
  echo "guest image ECR credential provider does not mint ECR login tokens" >&2
  exit 1
fi
if ! debugfs -R 'cat /opt/credential-provider/bin/ecr-credential-provider' "$guest_image_path" 2>/dev/null | grep -q '/bin/mktemp'; then
  echo "guest image ECR credential provider does not use absolute helper paths for mktemp" >&2
  exit 1
fi
if ! debugfs -R 'cat /opt/credential-provider/bin/ecr-credential-provider' "$guest_image_path" 2>/dev/null | grep -q '/bin/printf'; then
  echo "guest image ECR credential provider does not use absolute helper paths for printf" >&2
  exit 1
fi
if ! debugfs -R 'cat /etc/rancher/k3s/ecr-credential-provider.yaml' "$guest_image_path" 2>/dev/null | grep -q 'ecr-credential-provider'; then
  echo "guest image kubelet credential provider config is missing the ECR provider entry" >&2
  exit 1
fi
if [[ -z "$(debugfs -R 'cat /etc/port-k3s-version' "$guest_image_path" 2>/dev/null | tr -d '\r\n')" ]]; then
  echo "guest image K3s version marker is empty" >&2
  exit 1
fi
k3s_store_path="$(debugfs -R 'cat /etc/port-k3s-store-path' "$guest_image_path" 2>/dev/null | tr -d '\r\n')"
if [[ -z "$k3s_store_path" ]]; then
  echo "guest image K3s store marker is empty" >&2
  exit 1
fi
require_debugfs_path "$k3s_store_path/bin/k3s"
if ! debugfs -R 'cat /etc/hosts' "$guest_image_path" 2>/dev/null | grep -q '127.0.0.1 localhost'; then
  echo "guest image /etc/hosts does not include the localhost entry" >&2
  exit 1
fi
if [[ "$guest_image_path" == */x86_64/firecracker/* ]]; then
  initrd_path="$(dirname "$guest_image_path")/initrd.cpio.gz"
  if [[ ! -f "$initrd_path" ]]; then
    echo "guest image initrd is missing: $initrd_path" >&2
    exit 1
  fi
  if [[ "$(stat -c '%s' "$initrd_path")" -le 0 ]]; then
    echo "guest image initrd is empty: $initrd_path" >&2
    exit 1
  fi
fi

if [[ "$guest_image_path" == */x86_64/firecracker/standard/* ]]; then
  require_debugfs_path /etc/port-kernel-modules-release
  kernel_release="$(debugfs -R 'cat /etc/port-kernel-modules-release' "$guest_image_path" 2>/dev/null | tr -d '\r\n')"
  if [[ -z "$kernel_release" ]]; then
    echo "guest image kernel modules marker is empty" >&2
    exit 1
  fi
  require_debugfs_path "/lib/modules/${kernel_release}/modules.dep"
fi

if [[ "$guest_image_path" == */x86_64/firecracker/* ]]; then
  require_debugfs_path /etc/port-preloaded-images.txt
  preloaded_images="$(debugfs -R 'cat /etc/port-preloaded-images.txt' "$guest_image_path" 2>/dev/null || true)"
  if [[ -z "$(printf '%s\n' "$preloaded_images" | tr -d '\r\n[:space:]')" ]]; then
    echo "guest image preloaded image manifest is empty" >&2
    exit 1
  fi
  while IFS= read -r image; do
    [[ -n "$image" ]] || continue
    archive_name="$(printf '%s' "$image" | tr '/:@' '___').tar"
    require_debugfs_path "/var/lib/rancher/k3s/agent/images/${archive_name}"
  done <<EOF
$preloaded_images
EOF
fi

guest_image_size="$(stat -c '%s' "$guest_image_path")"
printf 'validated guest image: %s\n' "$guest_image_path"
printf 'guest image architecture: %s\n' "$expected_architecture"
printf 'guest image protection mode: %s\n' "$expected_protection_mode"
printf 'guest image K3s version: %s\n' "$(debugfs -R 'cat /etc/port-k3s-version' "$guest_image_path" 2>/dev/null | tr -d '\r\n')"
printf 'guest image bytes: %s\n' "$guest_image_size"

#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <output-path>" >&2
  exit 2
fi

output_path="$1"
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

resolve_pvm_build_flake_ref() {
  if [[ -n "${PORT_PVM_BUILD_FLAKE_REF:-}" ]]; then
    printf '%s\n' "$PORT_PVM_BUILD_FLAKE_REF"
    return 0
  fi

  local sibling_repo="${repo_root}/../pvm-builds"
  if [[ -f "${sibling_repo}/flake.nix" ]]; then
    printf 'git+file://%s\n' "$sibling_repo"
    return 0
  fi

  echo "missing PVM guest kernel build source: set PORT_PVM_BUILD_FLAKE_REF or provide a sibling ../pvm-builds checkout" >&2
  exit 1
}

case "$output_path" in
  */x86_64/firecracker/standard/*)
    arch="x86_64"
    kernel_source="$(nix build --option eval-cache false --no-link --print-out-paths nixpkgs#linuxPackages_latest.kernel.dev)"
    kernel_path="${kernel_source}/vmlinux"
    kernel_origin="nixpkgs#linuxPackages_latest.kernel.dev"
    ;;
  */x86_64/firecracker/pvm/*)
    arch="x86_64"
    pvm_build_flake_ref="$(resolve_pvm_build_flake_ref)"
    pvm_guest_kernel_attr="${PORT_PVM_GUEST_KERNEL_ATTR:-packages.x86_64-linux.linux-port-pvm-guest}"
    kernel_source="$(nix build --option eval-cache false --no-link --print-out-paths "${pvm_build_flake_ref}#${pvm_guest_kernel_attr}")"
    kernel_path="${kernel_source}/vmlinux"
    kernel_origin="${pvm_build_flake_ref}#${pvm_guest_kernel_attr}"
    ;;
  */aarch64/firecracker/standard/*)
    arch="aarch64"
    kernel_key="firecracker-ci/v1.14/aarch64/vmlinux-6.1.155"
    kernel_sha256="61baeae1ac6197be4fc5c71fa78df266acdc33c54570290d2f611c2b42c105be"
    ;;
  */aarch64/firecracker/pvm/*)
    echo "aarch64/firecracker/pvm remains research-only and has no build pipeline yet" >&2
    exit 1
    ;;
  *)
    echo "unsupported artifact selector for demo kernel pipeline: $output_path" >&2
    exit 1
    ;;
esac

mkdir -p "$(dirname "$output_path")"
if [[ -n "${kernel_path:-}" ]]; then
  if [[ ! -f "$kernel_path" ]]; then
    echo "missing Nix kernel artifact: $kernel_path" >&2
    exit 1
  fi
  actual_sha256="$(sha256sum "$kernel_path" | awk '{print $1}')"
  install -m 0644 "$kernel_path" "$output_path"
  printf 'kernel source: %s\n' "$kernel_origin"
  printf 'kernel store path: %s\n' "$kernel_source"
else
  kernel_url="http://spec.ccfc.min.s3.amazonaws.com/${kernel_key}"
  tmp_file="$(mktemp)"
  trap 'rm -f "$tmp_file"' EXIT
  curl -fsSL "$kernel_url" -o "$tmp_file"
  actual_sha256="$(sha256sum "$tmp_file" | awk '{print $1}')"

  if [[ "$actual_sha256" != "$kernel_sha256" ]]; then
    echo "kernel sha256 mismatch: expected $kernel_sha256 got $actual_sha256" >&2
    exit 1
  fi

  install -m 0644 "$tmp_file" "$output_path"
  printf 'kernel source: %s\n' "$kernel_url"
fi

if [[ "$output_path" == */firecracker/pvm/* ]]; then
  printf 'kernel protection mode: pvm\n'
else
  printf 'kernel protection mode: standard\n'
fi
printf 'kernel sha256: %s\n' "$actual_sha256"
printf 'kernel artifact: %s\n' "$output_path"

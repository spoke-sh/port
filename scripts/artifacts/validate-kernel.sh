#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <kernel-path>" >&2
  exit 2
fi

kernel_path="$1"
if [[ ! -f "$kernel_path" ]]; then
  echo "missing kernel artifact: $kernel_path" >&2
  exit 1
fi

case "$kernel_path" in
  */x86_64/firecracker/standard/*)
    arch="x86_64"
    expected_path="$(nix build --option eval-cache false --no-link --print-out-paths nixpkgs#linuxPackages_latest.kernel.dev)/vmlinux"
    ;;
  */x86_64/firecracker/pvm/*)
    arch="x86_64"
    expected_sha256="e41c7048bd2475e7e788153823fcb9166a7e0b78c4c443bd6446d015fa735f53"
    ;;
  */aarch64/firecracker/standard/*)
    arch="aarch64"
    expected_sha256="61baeae1ac6197be4fc5c71fa78df266acdc33c54570290d2f611c2b42c105be"
    ;;
  */aarch64/firecracker/pvm/*)
    echo "aarch64/firecracker/pvm remains research-only and has no validation pipeline yet" >&2
    exit 1
    ;;
  *)
    echo "unsupported artifact selector for demo kernel validation: $kernel_path" >&2
    exit 1
    ;;
esac

actual_sha256="$(sha256sum "$kernel_path" | awk '{print $1}')"
if [[ -n "${expected_path:-}" ]]; then
  if [[ ! -f "$expected_path" ]]; then
    echo "missing expected Nix kernel validation target: $expected_path" >&2
    exit 1
  fi
  expected_sha256="$(sha256sum "$expected_path" | awk '{print $1}')"
  if [[ "$actual_sha256" != "$expected_sha256" ]]; then
    echo "kernel sha256 mismatch: expected $expected_sha256 got $actual_sha256" >&2
    exit 1
  fi
else
  if [[ "$actual_sha256" != "$expected_sha256" ]]; then
    echo "kernel sha256 mismatch: expected $expected_sha256 got $actual_sha256" >&2
    exit 1
  fi
fi

kernel_size="$(stat -c '%s' "$kernel_path")"
if [[ "$kernel_size" -le 0 ]]; then
  echo "kernel artifact is empty: $kernel_path" >&2
  exit 1
fi

printf 'validated kernel: %s\n' "$kernel_path"
if [[ "$kernel_path" == */firecracker/pvm/* ]]; then
  printf 'kernel protection mode: pvm\n'
else
  printf 'kernel protection mode: standard\n'
fi
printf 'kernel sha256: %s\n' "$actual_sha256"
printf 'kernel bytes: %s\n' "$kernel_size"

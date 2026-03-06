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

arch="$(uname -m)"
case "$arch" in
  x86_64)
    expected_sha256="e41c7048bd2475e7e788153823fcb9166a7e0b78c4c443bd6446d015fa735f53"
    ;;
  aarch64)
    expected_sha256="61baeae1ac6197be4fc5c71fa78df266acdc33c54570290d2f611c2b42c105be"
    ;;
  *)
    echo "unsupported architecture for demo kernel validation: $arch" >&2
    exit 1
    ;;
esac

actual_sha256="$(sha256sum "$kernel_path" | awk '{print $1}')"
if [[ "$actual_sha256" != "$expected_sha256" ]]; then
  echo "kernel sha256 mismatch: expected $expected_sha256 got $actual_sha256" >&2
  exit 1
fi

kernel_size="$(stat -c '%s' "$kernel_path")"
if [[ "$kernel_size" -le 0 ]]; then
  echo "kernel artifact is empty: $kernel_path" >&2
  exit 1
fi

printf 'validated kernel: %s\n' "$kernel_path"
printf 'kernel sha256: %s\n' "$actual_sha256"
printf 'kernel bytes: %s\n' "$kernel_size"

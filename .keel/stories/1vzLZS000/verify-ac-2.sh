#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

nix develop -c cargo test -q -p port-runtime avf

tmp="$(mktemp /tmp/port-avf-doctor-XXXX.toml)"
trap 'rm -f "$tmp"' EXIT

cp examples/port.toml "$tmp"
perl -0pi -e 's/\[machines\.demo\]\n# Standard shipped lane:\n#   keep this machine on protection_mode = "standard" when you want the real\n#   local Firecracker launch proof while PVM remains a gated admission lane\.\nhost = "local"\nkernel = "demo-kernel"\nguest_image = "demo-guest"\nsubstrate = "firecracker"\nprotection_mode = "standard"\narchitecture = "native"/[machines.demo]\nhost = "mac-local"\nkernel = "demo-kernel"\nguest_image = "demo-guest"\nsubstrate = "avf"\nprotection_mode = "standard"\narchitecture = "x86_64"/s' "$tmp"
printf '%s\n' \
  '' \
  '[hosts.mac-local]' \
  'platform = "macos"' \
  'provider = "local"' \
  '' \
  '[hosts.mac-local.connection]' \
  'mode = "local"' \
  '' \
  '[hosts.mac-local.firecracker]' \
  'local_launch = false' \
  'notes = ["AVF local execution is modeled separately from Firecracker."]' \
  '' \
  '[[artifacts.kernels.demo-kernel.variants]]' \
  'path = "artifacts/kernel/demo/x86_64/avf/standard/vmlinux"' \
  '' \
  '[artifacts.kernels.demo-kernel.variants.selector]' \
  'architecture = "x86_64"' \
  'substrate = "avf"' \
  'protection_mode = "standard"' \
  '' \
  '[[artifacts.guest_images.demo-guest.variants]]' \
  'path = "artifacts/guest/demo/x86_64/avf/standard/rootfs.ext4"' \
  '' \
  '[artifacts.guest_images.demo-guest.variants.selector]' \
  'architecture = "x86_64"' \
  'substrate = "avf"' \
  'protection_mode = "standard"' \
  >>"$tmp"

output="$(nix develop -c cargo run -q -p port-cli -- --config "$tmp" doctor)"
printf '%s\n' "$output"

printf '%s\n' "$output" | grep -F "check[fail]: avf:demo:host-platform"
printf '%s\n' "$output" | grep -F "check[ok]: avf:demo:host-architecture"
printf '%s\n' "$output" | grep -F "check[fail]: avf:demo:runtime-availability"
printf '%s\n' "$output" | grep -F "note: macOS operators can run Port against the AVF lane locally;"

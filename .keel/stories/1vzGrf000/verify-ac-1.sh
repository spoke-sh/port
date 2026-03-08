#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port
nix develop -c cargo test -q -p port-runtime
nix develop -c cargo run -q -p port-cli -- --config /home/alex/workspace/spoke-sh/port/examples/port.toml artifacts build --artifact demo-kernel --architecture x86-64 --substrate firecracker --protection-mode pvm
nix develop -c cargo run -q -p port-cli -- --config /home/alex/workspace/spoke-sh/port/examples/port.toml artifacts validate --artifact demo-kernel --architecture x86-64 --substrate firecracker --protection-mode pvm
nix develop -c cargo run -q -p port-cli -- --config /home/alex/workspace/spoke-sh/port/examples/port.toml artifacts build --artifact demo-guest --architecture x86-64 --substrate firecracker --protection-mode pvm
nix develop -c cargo run -q -p port-cli -- --config /home/alex/workspace/spoke-sh/port/examples/port.toml artifacts validate --artifact demo-guest --architecture x86-64 --substrate firecracker --protection-mode pvm

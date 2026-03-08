#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

nix develop -c cargo run -q -p port-cli -- --help | rg -n "Native macOS AVF workflow|demo-avf|PORT_AVF_LAUNCHER"
nix develop -c cargo run -q -p port-cli -- --config examples/port.toml doctor | rg -n "avf:demo-avf:host-platform|avf:demo-avf:host-architecture|avf:demo-avf:runtime-availability"
rg -n "demo-avf|PORT_AVF_LAUNCHER|guest-agent.sock|console.log" README.md docs/avf.md docs/operators.md examples/port.toml

#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

nix develop -c cargo test -q -p port-model
nix develop -c cargo test -q -p port-runtime

doctor_json="$(nix develop -c cargo run -q -p port-cli -- --config examples/port.toml doctor --format json)"
printf '%s\n' "$doctor_json"

DOCTOR_JSON="$doctor_json" python3 - <<'PY'
import json
import os

report = json.loads(os.environ["DOCTOR_JSON"])
checks = {item["name"]: item for item in report["checks"]}

local = checks["pvm:local:x86_64:host-kit-contract"]
assert local["ok"], local
assert "firecracker-pvm" in local["detail"], local["detail"]
assert "PORT_PVM_FIRECRACKER_BINARY" in local["detail"], local["detail"]

aws = checks["pvm:aws-linux-node:x86_64:host-kit-contract"]
assert aws["ok"], aws
assert "firecracker-pvm" in aws["detail"], aws["detail"]
assert "PORT_PVM_FIRECRACKER_BINARY" in aws["detail"], aws["detail"]

generic = checks["pvm:generic-linux-node:x86_64:host-kit-contract"]
assert not generic["ok"], generic
assert "host-kit contract" in generic["detail"], generic["detail"]
PY

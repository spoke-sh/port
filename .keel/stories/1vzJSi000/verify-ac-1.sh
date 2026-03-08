#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

nix develop -c cargo test -q -p port-hosted-protocol
nix develop -c cargo test -q -p port-runtime node_agent_launches_pvm_machine_from_prepared_host_kit

#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

nix develop -c cargo test -q -p port-guest-agent service_manages_process_lifecycle_and_redacts_secret_output

#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"

nix develop -c cargo test -q -p port-runtime hosted_copy_stream_errors_include_route_context -- --exact

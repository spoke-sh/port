#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"

nix develop -c cargo test -q -p port-runtime hosted_copy_uses_stream_route_and_round_trips_bytes -- --exact
nix develop -c cargo test -q -p port-runtime control_plane_proxies_copy_stream_through_node_agent_guest_transport -- --exact

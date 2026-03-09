#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

nix develop -c cargo test -q -p port-hosted-protocol control_plane_routes_render_canonical_paths
nix develop -c cargo test -q -p port-hosted-protocol node_routes_render_internal_paths
nix develop -c cargo test -q -p port-sdk detached_forward_requests_use_canonical_hosted_paths

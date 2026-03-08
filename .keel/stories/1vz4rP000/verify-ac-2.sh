#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../../.."

nix eval .#devShells.x86_64-linux.default.drvPath --show-trace
nix develop -c bash -lc 'command -v firecracker && command -v ip && command -v iptables'

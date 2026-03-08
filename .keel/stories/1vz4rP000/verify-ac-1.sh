#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../../.."

nix eval .#devShells.aarch64-darwin.default.drvPath --show-trace

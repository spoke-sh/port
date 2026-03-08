#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../../.."

grep -nE 'macOS dev shell provides repo tooling only|nix develop.*macOS|Linux-only runtime packages' \
  flake.nix \
  README.md \
  docs/operators.md

#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== build exported keel package =="
nix build .#keel --no-link

echo
echo "== built keel version =="
keel_path="$(nix path-info .#keel)"
"$keel_path/bin/keel" --version

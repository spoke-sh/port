#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

grep -nE "port artifacts build|port artifacts validate|port artifacts push|port artifacts pull|prepare-pvm-node|imported-inventory.json" \
  README.md docs/pvm.md docs/hosted.md

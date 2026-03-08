#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../../.."

cargo test -q -p port-model

grep -nE 'FirecrackerPvmLaneContract|PvmHostKit|PvmArtifactKit|pti=off|artifact-variants' \
  crates/port-model/src/lib.rs

grep -nE 'x86_64 Host Kit Contract|x86_64 Artifact Kit Contract|pti=off|Future Port validation' \
  docs/pvm.md \
  docs/artifacts.md

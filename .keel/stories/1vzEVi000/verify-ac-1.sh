#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-hosted-protocol =="
cargo test -q -p port-hosted-protocol

echo
echo "== cargo test -q -p port-sdk =="
cargo test -q -p port-sdk

echo
echo "== cargo test -q -p port-model =="
cargo test -q -p port-model

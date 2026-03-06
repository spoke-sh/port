---
id: 1vyerj000
title: Model Cloud Linux Providers
type: feat
status: in-progress
created_at: 2026-03-06T15:45:55
updated_at: 2026-03-06T15:49:59
scope: 1vydg7000/1vyeq5000
started_at: 2026-03-06T15:49:59
---

# Model Cloud Linux Providers

## Summary

Extend the Port host model and canonical example config so remote Linux targets
carry explicit provider identity for generic Linux, AWS, GCP, and Azure
instead of relying on implicit SSH-only intent.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-01/AC-01] The canonical Port model distinguishes local Linux, generic remote Linux, AWS, GCP, and Azure host targets with explicit provider identity. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-model && rg -n "provider\\s*=\\s*\"(local|generic-linux|aws|gcp|azure)\"" /home/alex/workspace/spoke-sh/port/examples/port.toml', proof: ac-1.log-->

---
# system-managed
id: VFH8C1fHP
status: backlog
created_at: 2026-03-29T09:40:46
updated_at: 2026-03-29T09:41:57
# authored
title: Fix Packaged Guest Artifact Validation Contract
type: feat
operator-signal:
scope: VFH7YspJx/VFH7t3cG9
index: 3
---

# Fix Packaged Guest Artifact Validation Contract

## Summary

Make the shipped guest artifact validate path install-safe so `port artifacts
validate` works from the packaged CLI contract instead of resolving scripts
from source-build-only locations.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] `port --config examples/port.toml artifacts validate --artifact demo-guest --architecture x86-64` succeeds without looking for `validate-guest-image.sh` under `/build/...`. <!-- verify: manual, SRS-03:start:end -->

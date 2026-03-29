---
# system-managed
id: VFH8C1fHP
status: done
created_at: 2026-03-29T09:40:46
updated_at: 2026-03-29T11:20:49
# authored
title: Fix Packaged Guest Artifact Validation Contract
type: feat
operator-signal:
scope: VFH7YspJx/VFH7t3cG9
index: 3
started_at: 2026-03-29T11:12:14
submitted_at: 2026-03-29T11:20:44
completed_at: 2026-03-29T11:20:49
---

# Fix Packaged Guest Artifact Validation Contract

## Summary

Make the shipped guest artifact validate path install-safe so `port artifacts
validate` works from the packaged CLI contract instead of resolving scripts
from source-build-only locations.

## Acceptance Criteria

- [x] [SRS-03/AC-01] `port --config examples/port.toml artifacts validate --artifact demo-guest --architecture x86-64` succeeds without looking for `validate-guest-image.sh` under `/build/...`. Verified in `EVIDENCE/ac-1.log`, `EVIDENCE/ac-1.nix-package-validate.log`, `EVIDENCE/ac-2.package-proof.log`, and `EVIDENCE/ac-3.prefix-validate.log`. <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->

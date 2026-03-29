---
# system-managed
id: VFHn1Pslh
status: backlog
created_at: 2026-03-29T12:22:56
updated_at: 2026-03-29T12:24:38
# authored
title: Verify Unchanged Downstream Infra GitOps Handoff
type: feat
operator-signal:
scope: VFHmKH5XR/VFHmctWC5
index: 4
---

# Verify Unchanged Downstream Infra GitOps Handoff

## Summary

Verify that downstream `spoke infra` can consume Port's local cluster handoff
unchanged through its local bootstrap and health flows once the real K3s
runtime and GitOps prerequisites are in place.

## Acceptance Criteria

- [ ] [SRS-05/AC-01] Downstream `infra bootstrap --env local` succeeds unchanged against the Port-provided cluster handoff. <!-- verify: manual, SRS-05:start:end -->
- [ ] [SRS-NFR-03/AC-02] Downstream `infra health --env local` succeeds unchanged while this voyage remains explicitly local-only and single-node only. <!-- verify: manual, SRS-NFR-03:start:end -->

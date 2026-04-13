---
# system-managed
id: VGcgtFI9v
status: done
created_at: 2026-04-12T16:39:11
updated_at: 2026-04-12T17:45:19
# authored
title: Record Hosted Worker Stability Soak Proof
type: feat
operator-signal:
scope: VGcgU9T57/VGcghwZrb
index: 3
started_at: 2026-04-12T17:37:06
submitted_at: 2026-04-12T17:45:16
completed_at: 2026-04-12T17:45:19
---

# Record Hosted Worker Stability Soak Proof

## Summary

Record reviewable proof that hosted workers remain healthy or recover correctly
across the observed 60-90 minute drift window.

## Acceptance Criteria

- [x] [SRS-03/AC-01] Port records reviewable proof that hosted workers remain healthy or recover correctly across the targeted drift window. <!-- verify: manual, SRS-03:start:end, proof: ac-1.gif -->
- [x] [SRS-NFR-02/AC-02] The stability proof remains reviewable without workstation-local lore. <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-2.log -->

## Proof

- AC-01: `EVIDENCE/ac-1.gif`, `EVIDENCE/ac-1.log`, and `EVIDENCE/hosted-k3s-ha-failover-workflow.cast` record the hosted HA/failover proof slice, showing `cloud-aws-worker` still visible as `Ready` before and after a simulated primary control-plane guest replacement.
- AC-02: `EVIDENCE/ac-2.log` records the committed proof harness structure, including the temporary hosted control-plane isolation, explicit worker placement, stability checkpoints, and replacement step so the artifact stays reviewable without relying on workstation-local setup lore.

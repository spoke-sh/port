---
# system-managed
id: VGcgtAKBo
status: done
created_at: 2026-04-12T16:39:11
updated_at: 2026-04-12T17:15:21
# authored
title: Document Downstream Hosted Status Contract
type: feat
operator-signal:
scope: VGcgU7q58/VGcghuutu
index: 3
started_at: 2026-04-12T17:12:57
submitted_at: 2026-04-12T17:15:13
completed_at: 2026-04-12T17:15:21
---

# Document Downstream Hosted Status Contract

## Summary

Author the downstream hosted status contract and its proof posture so paired
infra work can consume Port truth intentionally instead of by reverse
engineering runtime output.

## Acceptance Criteria

- [x] [SRS-03/AC-01] The downstream hosted status contract is documented with the fields and semantics that consumers may rely on. <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
- [x] [SRS-03/AC-02] The proof posture for validating the hosted status contract is documented alongside the contract. <!-- verify: manual, SRS-03:start:end, proof: ac-2.log-->

## Proof

- AC-01: `EVIDENCE/ac-1.log` records the manual review of [CONTRACT.md](../../epics/VGcgU7q58/voyages/VGcghuutu/CONTRACT.md), which now defines the canonical hosted status JSON surface, stable downstream fields, and the best-effort diagnostic boundary.
- AC-02: `EVIDENCE/ac-2.log` records the manual review of the `Proof Posture` section in [CONTRACT.md](../../epics/VGcgU7q58/voyages/VGcghuutu/CONTRACT.md), plus the linked voyage docs, which now document how the hosted status contract should be validated.

---
# system-managed
id: VFgcoUoUd
status: backlog
created_at: 2026-04-02T18:19:12
updated_at: 2026-04-02T18:20:56
# authored
title: Publish Hosted AWS PVM Operator Proof
type: feat
operator-signal:
scope: VFgcPDfEj/VFgclbQzC
index: 1
---

# Publish Hosted AWS PVM Operator Proof

## Summary

Publish the canonical operator proof for the hosted AWS PVM lane so Port shows
how to prepare the node and then launch, inspect, and stop `cloud-aws` on the
live hosted runtime path.

## Acceptance Criteria

<!-- verify: manual, SRS-03:start:end -->
- [ ] [SRS-03/AC-01] Port publishes a canonical hosted AWS PVM proof that runs `prepare-pvm-node` plus `machine launch`, `status`, and `stop` for `cloud-aws` on a prepared x86_64 AWS node. <!-- [SRS-03/AC-01] verify: manual -->
<!-- verify: manual, SRS-NFR-02:start:end -->
- [ ] [SRS-NFR-02/AC-02] The proof and operator-facing docs keep the scope boundary explicit: x86_64 AWS hosted PVM only, with provider-aware prerequisites and failure expectations. <!-- [SRS-NFR-02/AC-02] verify: manual -->

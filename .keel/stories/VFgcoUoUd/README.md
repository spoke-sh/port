---
# system-managed
id: VFgcoUoUd
status: done
created_at: 2026-04-02T18:19:12
updated_at: 2026-04-02T19:17:29
# authored
title: Publish Hosted AWS PVM Operator Proof
type: feat
operator-signal:
scope: VFgcPDfEj/VFgclbQzC
index: 1
started_at: 2026-04-02T19:08:10
completed_at: 2026-04-02T19:17:29
---

# Publish Hosted AWS PVM Operator Proof

## Summary

Publish the canonical operator proof for the hosted AWS PVM lane so Port shows
how to prepare the node and then launch, inspect, and stop `cloud-aws` on the
live hosted runtime path.

## Acceptance Criteria

<!-- verify: command, SRS-03:start:end, proof: ac-1.gif -->
- [x] [SRS-03/AC-01] Port publishes a canonical hosted AWS PVM proof that runs `prepare-pvm-node` plus `machine launch`, `status`, and `stop` for `cloud-aws` on a prepared x86_64 AWS node. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && ./scripts/render-hosted-pvm-proof.sh .keel/stories/VFgcoUoUd/EVIDENCE', proof: ac-1.gif -->
<!-- verify: command, SRS-NFR-02:start:end, proof: ac-2.log -->
- [x] [SRS-NFR-02/AC-02] The proof and operator-facing docs keep the scope boundary explicit: x86_64 AWS hosted PVM only, with provider-aware prerequisites and failure expectations. <!-- [SRS-NFR-02/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "hosted-pvm-demo|render-hosted-pvm-proof|x86_64|AWS|aarch64|GCP|Azure|prepare-pvm-node" README.md docs/operators.md docs/hosted.md docs/pvm.md docs/cloud.md', proof: ac-2.log -->

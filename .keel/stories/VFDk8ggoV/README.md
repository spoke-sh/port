---
# system-managed
id: VFDk8ggoV
status: done
created_at: 2026-03-28T19:46:24
updated_at: 2026-03-29T08:34:33
# authored
title: Publish Cluster Operator Contract And Infra Handoff Proof
type: feat
operator-signal:
scope: VFDhlRjOf/VFDk8fdnG
index: 4
started_at: 2026-03-29T08:18:17
completed_at: 2026-03-29T08:34:33
---

<!-- verify: command, SRS-04:start:end, proof: ac-1.log -->
<!-- verify: command, SRS-NFR-02:start:end, proof: ac-2.gif -->

# Publish Cluster Operator Contract And Infra Handoff Proof

## Summary

Publish the new local cluster operator contract, make the thin downstream infra
handoff explicit, and record a human-reviewable proof for the first cluster
workflow.

## Acceptance Criteria

- [x] [SRS-04/AC-01] Docs and help publish the thin infra handoff and remove raw machine or guest choreography as the blessed cluster workflow. <!-- [SRS-04/AC-01] verify: nix develop --command bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo run -q -p port-cli -- --help | rg -n "cluster show --cluster demo|cluster up --cluster demo|cluster kubeconfig --cluster demo" && rg -n "Local cluster first slice|Port owns machine launch|Downstream infra asks Port|not the blessed cluster workflow|cluster status is Port.s answer|cluster kubeconfig --format json|render-local-cluster-proof" README.md docs/operators.md CONFIGURATION.md && ! rg -n "Hosted Stateless K3s First Slice|curl https://get.k3s.io" README.md docs/operators.md CONFIGURATION.md', proof: ac-1.log -->
- [x] [SRS-NFR-02/AC-02] The story records one human-reviewable proof artifact for the canonical local cluster workflow. <!-- [SRS-NFR-02/AC-02] verify: nix develop --command bash -lc 'cd /home/alex/workspace/spoke-sh/port && ./scripts/render-local-cluster-proof.sh .keel/stories/VFDk8ggoV/EVIDENCE', proof: ac-2.gif -->

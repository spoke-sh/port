---
id: VDfzOEeFL
title: Publish Hosted K3s Operator Proof
type: feat
status: backlog
created_at: 2026-03-12T10:44:50
updated_at: 2026-03-12T10:46:00
operator-signal: 
scope: VDcStSMlp/VDfytSpPs
index: 2
---

# Publish Hosted K3s Operator Proof

## Summary

Publish the first hosted K3s operator workflow in canonical docs and record a
human-reviewable proof artifact for cluster bring-up and review.

## Acceptance Criteria

<!-- verify: command, SRS-05:start:end, proof: ac-1.log -->
- [ ] [SRS-05/AC-01] The canonical docs publish the hosted stateless K3s contract, workflow, and first-slice boundaries without inventing a second Kubernetes-only toolchain. <!-- [SRS-05/AC-01] verify: sh -c 'cd /home/alex/workspace/spoke-sh/port && rg -n "k3s|kubernetes|hosted" README.md docs CONFIGURATION.md', proof: ac-1.log -->
<!-- verify: command, SRS-NFR-03:start:end, proof: ac-2.gif -->
- [ ] [SRS-NFR-03/AC-02] The story records at least one human-reviewable proof artifact through the proof system for the hosted K3s workflow. <!-- [SRS-NFR-03/AC-02] verify: sh -c 'cd /home/alex/workspace/spoke-sh/port && ./scripts/render-hosted-k3s-proof.sh .keel/stories/VDfzOEeFL/EVIDENCE', proof: ac-2.gif -->

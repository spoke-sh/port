---
id: VDeuzbve3
title: Publish Hybrid Execution Operator Proof
type: feat
status: backlog
created_at: 2026-03-12T06:21:06
updated_at: 2026-03-12T06:23:42
operator-signal: 
scope: VDcStPolu/VDeuazAgk
index: 3
---

# Publish Hybrid Execution Operator Proof

## Summary

Publish the hybrid local, hosted, and SSH operator contract in docs and record
the first human-reviewable proof artifact for the SSH-first workflow.

## Acceptance Criteria

- [ ] [SRS-05/AC-01] The canonical docs publish the hybrid execution contract and the first SSH-first operator workflow without inventing a second command family. <!-- [SRS-05/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n \"ssh|hosted-control-plane|hybrid\" README.md docs', proof: ac-1.log -->
- [ ] [SRS-NFR-03/AC-02] The story records at least one human-reviewable proof artifact through the proof system for the SSH-first workflow. <!-- [SRS-NFR-03/AC-02] verify: vhs .keel/stories/VDeuzbve3/EVIDENCE/hybrid-ssh-workflow.tape, proof: ac-2.gif -->

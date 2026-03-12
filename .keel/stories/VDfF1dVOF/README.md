---
id: VDfF1dVOF
title: Publish Attached Volume Operator Proof
type: feat
status: done
created_at: 2026-03-12T07:40:40
updated_at: 2026-03-12T08:33:39
operator-signal: 
scope: VDcStQqlo/VDfEyGkVf
index: 3
started_at: 2026-03-12T08:30:45
completed_at: 2026-03-12T08:33:39
---

<!-- verify: command, SRS-05:start:end, proof: ac-1.log -->
<!-- verify: command, SRS-NFR-03:start:end, proof: ac-2.gif -->

# Publish Attached Volume Operator Proof

## Summary

Publish the canonical attached-volume operator workflow in docs and record a
human-reviewable proof artifact for mission and story review.

## Acceptance Criteria

- [x] [SRS-05/AC-01] The canonical docs publish the attached-volume contract and the first direct-runtime operator workflow without inventing a second storage command family. <!-- [SRS-05/AC-01] verify: sh -c 'cd /home/alex/workspace/spoke-sh/port && /home/alex/.nix-profile/bin/rg -n "volume|attachment|storage" README.md docs CONFIGURATION.md', proof: ac-1.log -->
- [x] [SRS-NFR-03/AC-02] The story records at least one human-reviewable proof artifact through the proof system for the attached-volume workflow. <!-- [SRS-NFR-03/AC-02] verify: sh -c 'cd /home/alex/workspace/spoke-sh/port && ./scripts/render-attached-volume-proof.sh .keel/stories/VDfF1dVOF/EVIDENCE', proof: ac-2.gif -->

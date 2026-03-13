---
id: VDi3O5dlc
title: Wire Repo-Level Screen Surface To App Hosting Proof
type: feat
status: done
created_at: 2026-03-12T19:13:16
updated_at: 2026-03-12T19:41:56
operator-signal: 
scope: VDi2y6gch/VDi3LHFpb
index: 1
started_at: 2026-03-12T19:35:39
completed_at: 2026-03-12T19:41:56
---

# Wire Repo-Level Screen Surface To App Hosting Proof

## Summary

Wire the current repo-level proof surface to show the hosted app proof as the
primary operator-facing evidence for this mission, using
`scripts/hosted-http-app-demo.sh` as the runnable workflow and
`scripts/render-hosted-http-app-proof.sh` as the recording path.

## Acceptance Criteria

<!-- verify: command, SRS-03:start:end, proof: ac-1.log -->
- [x] [SRS-03/AC-01] The current repo-level proof entrypoint surfaces the canonical hosted app proof workflow, including the runnable proof path and the recorded artifact, as the primary evidence for this slice. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && ./scripts/render-hosted-http-app-proof.sh .keel/stories/VDi3O5dlc/EVIDENCE >/dev/null && bash scripts/mission-report.sh VDi2jvg4P', proof: ac-1.log -->
<!-- verify: command, SRS-NFR-01:start:end, proof: ac-2.gif -->
- [x] [SRS-NFR-01/AC-01] The story records a human-reviewable artifact for the canonical hosted app proof through the repository proof system. <!-- [SRS-NFR-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && ./scripts/render-hosted-http-app-proof.sh .keel/stories/VDi3O5dlc/EVIDENCE', proof: ac-2.gif -->

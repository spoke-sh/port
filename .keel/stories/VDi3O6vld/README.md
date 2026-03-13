---
id: VDi3O6vld
title: Publish App Hosting Proof Contract And Boundaries
type: feat
status: done
created_at: 2026-03-12T19:13:16
updated_at: 2026-03-12T19:46:21
operator-signal: 
scope: VDi2y6gch/VDi3LHFpb
index: 2
started_at: 2026-03-12T19:44:19
completed_at: 2026-03-12T19:46:21
---

# Publish App Hosting Proof Contract And Boundaries

## Summary

Publish the canonical app-hosting proof contract, prerequisites, and explicit
boundaries so the first proof slice does not imply broader hosted guarantees
than Port currently ships.

## Acceptance Criteria

<!-- verify: command, SRS-04:start:end, proof: ac-1.log -->
- [x] [SRS-04/AC-01] README and focused docs describe the canonical hosted app proof path, its prerequisites, and its relationship to the current repo-level proof entrypoint. <!-- [SRS-04/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "just mission|scripts/hosted-http-app-demo.sh|scripts/render-hosted-http-app-proof.sh|PORT_DEMO_TOKEN|repo-level review surface" README.md docs/operators.md', proof: ac-1.log -->
<!-- verify: command, SRS-04:start:end, proof: ac-2.log -->
- [x] [SRS-04/AC-02] The published boundaries keep future `screen` naming work and future `atxt` recorder migration explicit as follow-on work instead of implying they shipped in this slice. <!-- [SRS-04/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "screen|atxt|follow-on|current repo-level entrypoint name is" README.md docs/operators.md', proof: ac-2.log -->
<!-- verify: command, SRS-NFR-01:start:end, proof: ac-3.log -->
- [x] [SRS-NFR-01/AC-02] The docs story links the human-reviewable proof path and artifact review expectations clearly enough for a maintainer to audit the workflow without reading implementation code first. <!-- [SRS-NFR-01/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "human-reviewable artifact|review surface|review artifact|just mission" README.md docs/operators.md', proof: ac-3.log -->

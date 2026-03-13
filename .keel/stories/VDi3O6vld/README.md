---
id: VDi3O6vld
title: Publish App Hosting Proof Contract And Boundaries
type: feat
status: backlog
created_at: 2026-03-12T19:13:16
updated_at: 2026-03-12T19:17:30
operator-signal: 
scope: VDi2y6gch/VDi3LHFpb
index: 2
---

# Publish App Hosting Proof Contract And Boundaries

## Summary

Publish the canonical app-hosting proof contract, prerequisites, and explicit
boundaries so the first proof slice does not imply broader hosted guarantees
than Port currently ships.

## Acceptance Criteria

<!-- verify: manual, SRS-04:start:end -->
- [ ] [SRS-04/AC-01] README and focused docs describe the canonical hosted app proof path, its prerequisites, and its relationship to the current repo-level proof entrypoint. <!-- [SRS-04/AC-01] verify: manual, proof: ac-1.log -->
<!-- verify: manual, SRS-04:start:end -->
- [ ] [SRS-04/AC-02] The published boundaries keep future `screen` naming work and future `atxt` recorder migration explicit as follow-on work instead of implying they shipped in this slice. <!-- [SRS-04/AC-02] verify: manual, proof: ac-2.log -->
<!-- verify: manual, SRS-NFR-01:start:end -->
- [ ] [SRS-NFR-01/AC-02] The docs story links the human-reviewable proof path and artifact review expectations clearly enough for a maintainer to audit the workflow without reading implementation code first. <!-- [SRS-NFR-01/AC-02] verify: manual, proof: ac-3.log -->

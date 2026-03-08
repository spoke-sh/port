---
id: 1vz4hB000
title: Define Hosted Node Inventory Model
type: feat
status: done
created_at: 2026-03-07T19:20:45
updated_at: 2026-03-07T19:39:19
scope: 1vz4Yn000/1vz4cU000
started_at: 2026-03-07T19:35:16
submitted_at: 2026-03-07T19:39:16
completed_at: 2026-03-07T19:39:19
---

# Define Hosted Node Inventory Model

## Summary

Define the first hosted node and host-group inventory contract so Port can map
placement, ownership, and later scheduling work onto shared machine vocabulary.

## Acceptance Criteria

<!-- verify: manual, SRS-02:start:end, proof: ac-1.log, ac-2.log -->
- [x] [SRS-02/AC-01] Port publishes implementation-ready node and host-group contracts in the shared model, including ownership, placement, and capability fields needed for hosted machine lifecycle routing. <!-- [SRS-02/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz4hB000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-02/AC-02] Hosted docs explain how nodes and host groups relate to later scheduler, monitoring, and services work without implying those features are already shipped. <!-- [SRS-02/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz4hB000/verify-ac-2.sh, proof: ac-2.log -->

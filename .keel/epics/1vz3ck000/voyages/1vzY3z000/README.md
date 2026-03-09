---
id: 1vzY3z000
title: PVM Host Kit And Artifact Delivery
status: in-progress
epic: 1vz3ck000
created_at: 2026-03-09T03:42:15
index: 5
updated_at: 2026-03-09T03:45:44
started_at: 2026-03-09T03:46:18
---

# PVM Host Kit And Artifact Delivery

> Make the x86_64 Firecracker/PVM lane reproducible and operable through canonical artifact build, pull, push, validate, and hosted node-preparation workflows.

## Documents

<!-- BEGIN DOCUMENTS -->
| Document | Description |
|----------|-------------|
| [SRS.md](SRS.md) | Requirements and verification criteria |
| [SDD.md](SDD.md) | Architecture and implementation details |
<!-- END DOCUMENTS -->

## Stories

<!-- BEGIN GENERATED -->
**Progress:** 2/4 stories complete

| Title | Type | Status |
|-------|------|--------|
| [Define Pvm Host Kit Package Contract](../../../../stories/1vzY51000/README.md) | feat | done |
| [Add Pvm Artifact Mobility Workflow](../../../../stories/1vzY52000/README.md) | feat | done |
| [Implement Hosted Pvm Node Preparation](../../../../stories/1vzY6F000/README.md) | feat | backlog |
| [Publish Pvm Host Kit Operator Workflow](../../../../stories/1vzY6J000/README.md) | feat | backlog |
<!-- END GENERATED -->

## Scope Summary

This voyage turns the prepared-node PVM lane into a reproducible delivery lane.
The emphasis is not on broadening the substrate matrix; it is on making the
existing `x86_64` Firecracker/PVM path operable through canonical Port
surfaces:

- explicit PVM host-kit artifacts and contracts
- first-class build, validate, push, and pull flows for PVM variants
- hosted node preparation and inventory import that advertises real PVM kits
- operator documentation and CLI proofs that match the shipped behavior

## Verification Plan

- Rust unit tests for model, runtime, and CLI behavior
- CLI proofs for `port artifacts ...`, hosted node preparation, and doctor
  output
- Repo-local verification scripts where multi-step proofs need deterministic
  roots

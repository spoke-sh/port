---
id: VDaiNwEwJ
title: Publish Foundational Docs And Simplify Operator Help
type: feat
status: done
created_at: 2026-03-11T13:05:56
updated_at: 2026-03-11T13:52:12
scope: VDaiFfFPe/VDaiL5HDr
index: 1
started_at: 2026-03-11T13:24:11
completed_at: 2026-03-11T13:52:12
---

# Publish Foundational Docs And Simplify Operator Help

## Summary

Publish root-level documentation contracts for Port, simplify top-level help
and README examples, and replace stale cargo-runner examples with canonical
`port` commands.

## Acceptance Criteria

<!-- verify: command, SRS-03:start:end, proof: ac-1.log -->
- [x] [SRS-03/AC-01] Port publishes root-level `CONSTITUTION.md`, `ARCHITECTURE.md`, `CONFIGURATION.md`, `RELEASE.md`, and `EVALUATIONS.md` docs that match the current product contract and are linked from the README. <!-- [SRS-03/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/VDaiNwEwJ/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-04:start:end, proof: ac-2.log -->
- [x] [SRS-04/AC-02] `port --help` and the README keep only 2-3 useful examples and direct detailed workflows to `CONFIGURATION.md` and focused docs. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/VDaiNwEwJ/verify-ac-2.sh, proof: ac-2.log -->
<!-- verify: command, SRS-05:start:end, proof: ac-3.log -->
- [x] [SRS-05/AC-03] User-facing docs and help replace `cargo run -p port-cli` examples with the canonical `port` command surface. <!-- [SRS-05/AC-03] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/VDaiNwEwJ/verify-ac-3.sh, proof: ac-3.log -->

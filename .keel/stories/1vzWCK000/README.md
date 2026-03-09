---
id: 1vzWCK000
title: Publish Hosted Artifact Mobility Workflow
type: feat
status: done
created_at: 2026-03-09T01:42:44
updated_at: 2026-03-09T02:46:21
scope: 1vzW8e000/1vzW9Q000
started_at: 2026-03-09T02:30:22
completed_at: 2026-03-09T02:46:21
---

# Publish Hosted Artifact Mobility Workflow

## Summary

Publish the first hosted artifact mobility workflow through README, artifact
docs, CLI help, and executable proof so operators can build, push, remove, and
pull a selected artifact variant end-to-end while understanding that OCI
support remains follow-on work.

## Acceptance Criteria

<!-- verify: command, SRS-04:start:end, proof: ac-1.log -->
- [x] [SRS-04/AC-01] Repo-local proof builds a selected artifact variant, pushes it to the hosted backend, removes the local output, then pulls the same variant back successfully through the canonical CLI. <!-- [SRS-04/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-cli --test artifact_commands cli_artifact_build_push_and_pull_round_trip_through_hosted_backend -- --exact', proof: ac-2.log -->
<!-- verify: command, SRS-05:start, proof: ac-3.log -->
- [x] [SRS-05/AC-01] README, `docs/artifacts.md`, and relevant CLI help publish the hosted artifact workflow, control-plane store ownership, and auth expectations while explicitly stating that OCI remains follow-on work. <!-- [SRS-05/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-cli tests::help_includes_primary_surfaces -- --exact && rg -n hosted-api README.md docs/artifacts.md crates/port-cli/src/lib.rs && rg -n PORT_DEMO_TOKEN README.md docs/artifacts.md crates/port-cli/src/lib.rs && rg -n follow-on README.md docs/artifacts.md crates/port-cli/src/lib.rs && rg -n .port/hosted README.md docs/artifacts.md crates/port-cli/src/lib.rs', proof: ac-2.log -->
<!-- verify: command, SRS-05:end, proof: ac-3.log -->
- [x] [SRS-05/AC-02] The voyage closes with recorded board evidence and verification for the shipped hosted backend rather than leaving `hosted-api` as a modeled-only placeholder. <!-- [SRS-05/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime -p port-cli && keel doctor && test -f .keel/stories/1vzWCK000/EVIDENCE/ac-1.log && test -f .keel/stories/1vzWCK000/EVIDENCE/ac-2.log && test -f .keel/stories/1vzWCK000/EVIDENCE/ac-3.log', proof: ac-3.log -->

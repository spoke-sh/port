---
id: 1vzY6J000
title: Publish Pvm Host Kit Operator Workflow
type: feat
status: backlog
created_at: 2026-03-09T03:44:39
updated_at: 2026-03-09T03:45:44
scope: 1vz3ck000/1vzY3z000
---

# Publish Pvm Host Kit Operator Workflow

## Summary

Publish the canonical operator workflow for PVM host-kit packaging, artifact
mobility, hosted node preparation, and proof so the lane is discoverable
through README, PVM docs, hosted docs, and CLI help.

## Acceptance Criteria

<!-- verify: command, SRS-04:start -->
- [ ] [SRS-04/AC-01] README, `docs/pvm.md`, `docs/hosted.md`, and CLI help publish the canonical `x86_64` Firecracker/PVM host-kit and artifact workflow, including the explicit `aarch64` research-only boundary. <!-- [SRS-04/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-cli help_includes_machine_commands_examples -- --exact && rg -n \"host kit|research-only|aarch64\" README.md docs/pvm.md docs/hosted.md crates/port-cli/src/lib.rs', proof: ac-1.log -->
<!-- verify: command, SRS-04:end -->
<!-- verify: command, SRS-04:start -->
- [ ] [SRS-04/AC-02] The published workflow includes repo-local proof commands for artifact build/validate/push/pull plus hosted node preparation/import so operators can verify the lane without hidden steps. <!-- [SRS-04/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n \"port artifacts build|port artifacts validate|port artifacts push|port artifacts pull|node\" README.md docs/pvm.md docs/hosted.md', proof: ac-2.log -->
<!-- verify: command, SRS-04:end -->

---
id: 1vzdMb000
title: Publish Cloud Hypervisor Operator Workflow
type: feat
status: done
scope: 1vzdKB000/1vzdKx000
created_at: 2026-03-09T09:21:49
updated_at: 2026-03-09T10:31:32
started_at: 2026-03-09T10:17:16
completed_at: 2026-03-09T10:31:32
---

# Publish Cloud Hypervisor Operator Workflow

## Summary

Publish the canonical local and hosted Cloud Hypervisor workflow through
README, cloud/operator docs, examples, and CLI help so the substrate is
discoverable and learnable from Port itself.

## Acceptance Criteria

<!-- verify: command, SRS-05:start, proof: ac-1.log -->
- [x] [SRS-05/AC-01] README, `docs/cloud.md`, `docs/operators.md`, examples, and CLI help publish one coherent Cloud Hypervisor workflow with explicit local and hosted proof commands. <!-- [SRS-05/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "Cloud Hypervisor|cloud-hypervisor" README.md docs/cloud.md docs/operators.md examples/port.toml crates/port-cli/src/lib.rs', proof: ac-1.log -->
<!-- verify: command, SRS-05:end -->
<!-- verify: command, SRS-05:start, proof: ac-2.log -->
- [x] [SRS-05/AC-02] The published operator surface keeps unsupported boundaries explicit and points at concrete verification evidence for the shipped Cloud Hypervisor lane. <!-- [SRS-05/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-cli -- --test-threads=1', proof: ac-2.log -->
<!-- verify: command, SRS-05:end -->

---
id: 1vydgm000
title: Document Operator Workflows
type: feat
status: in-progress
created_at: 2026-03-06T14:30:32
updated_at: 2026-03-06T15:39:25
scope: 1vydg7000/1vydgL000
started_at: 2026-03-06T15:39:25
---

# Document Operator Workflows

## Summary

Document the supported Linux, macOS, and Windows operator workflows and make
the CLI surface reflect those platform constraints instead of leaving them
implicit.

## Acceptance Criteria

<!-- verify: manual, SRS-07:start:end, proof: ac-1.log-->
- [x] [SRS-07/AC-01] README and supporting docs explain the Linux local-launch workflow end-to-end using canonical CLI commands. <!-- [SRS-07/AC-01] verify: rg -n "Linux Local Workflow|artifacts build --artifact demo-kernel|machine launch --machine demo" /home/alex/workspace/spoke-sh/port/README.md /home/alex/workspace/spoke-sh/port/docs/operators.md, proof: ac-2.log-->
- [x] [SRS-07/AC-02] macOS operator guidance explains the supported remote-host workflow and explicitly states why local Firecracker launch is unsupported. <!-- [SRS-07/AC-02] verify: rg -n "macOS|Linux host|unsupported" /home/alex/workspace/spoke-sh/port/README.md /home/alex/workspace/spoke-sh/port/docs/operators.md, proof: ac-3.log-->
- [x] [SRS-07/AC-03] Windows operator guidance explains the supported Linux or WSL-host workflow and explicitly states current constraints. <!-- [SRS-07/AC-03] verify: rg -n "Windows|WSL|/dev/kvm|remote Linux host" /home/alex/workspace/spoke-sh/port/README.md /home/alex/workspace/spoke-sh/port/docs/operators.md, proof: ac-4.log-->
- [x] [SRS-07/AC-04] CLI help and diagnostics align with the documented platform support matrix. <!-- [SRS-07/AC-04] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && nix develop -c cargo run -p port-cli -- --help && nix develop -c cargo run -p port-cli -- doctor', proof: ac-5.log-->

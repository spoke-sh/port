---
id: 1vydgm000
title: Document Operator Workflows
type: feat
status: icebox
created_at: 2026-03-06T14:30:32
updated_at: 2026-03-06T14:32:50
scope: 1vydg7000/1vydgL000
---

# Document Operator Workflows

## Summary

Document the supported Linux, macOS, and Windows operator workflows and make
the CLI surface reflect those platform constraints instead of leaving them
implicit.

## Acceptance Criteria

- [ ] [SRS-07/AC-01] README and supporting docs explain the Linux local-launch workflow end-to-end using canonical CLI commands.
- [ ] [SRS-07/AC-02] macOS operator guidance explains the supported remote-host workflow and explicitly states why local Firecracker launch is unsupported.
- [ ] [SRS-07/AC-03] Windows operator guidance explains the supported Linux or WSL-host workflow and explicitly states current constraints.
- [ ] [SRS-NFR-02/AC-01] CLI help and diagnostics align with the documented platform support matrix.

---
id: 1vzHRy000
title: Publish Pvm Admission Workflow
type: feat
status: backlog
created_at: 2026-03-08T09:57:54
updated_at: 2026-03-08T09:59:34
scope: 1vz3ck000/1vzHPo000
---

# Publish Pvm Admission Workflow

## Summary

Document the canonical local and hosted PVM admission workflow so operators can
discover what Port can prove today, what requires a prepared host kit, and what
remains explicitly unsupported.

## Acceptance Criteria

<!-- verify: command, SRS-04:start:end, proof: ac-1.log, ac-2.log -->
- [ ] [SRS-04/AC-01] README, `docs/pvm.md`, sample config, and CLI help explain local and hosted PVM admission, required host-kit prerequisites, and the explicit `aarch64` boundary. <!-- [SRS-04/AC-01] verify: cargo run -q -p port-cli -- --help, proof: ac-1.log -->
<!-- verify: command, SRS-04:start:end, proof: ac-2.log -->
- [ ] [SRS-04/AC-02] Repository-local proof commands or scripts demonstrate both the PVM admission path and the preserved standard Firecracker path with recorded evidence. <!-- [SRS-04/AC-02] verify: cargo test -q -p port-cli && cargo test -q -p port-runtime, proof: ac-2.log -->

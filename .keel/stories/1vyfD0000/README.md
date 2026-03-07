---
id: 1vyfD0000
title: Fix Help Example Guidance
type: feat
status: backlog
created_at: 2026-03-06T16:07:54
updated_at: 2026-03-06T16:08:42
scope: 1vydg7000/1vyfCm000
---

# Fix Help Example Guidance

## Summary

Make the `port --help` examples explicit about their environment prerequisites
and runnable sequence, then align the supporting docs so operators understand
when `nix develop` and `port doctor` are required.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [ ] [SRS-01/AC-01] `port --help` states the prerequisite environment for the local artifact and launch examples and presents a runnable local workflow order. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && /tmp/port-target/debug/port --help | rg -n "nix develop|port doctor|machine launch|artifacts build"', proof: ac-1.log-->
<!-- verify: manual, SRS-02:start:end, proof: ac-2.log-->
- [ ] [SRS-02/AC-01] README and operator docs explain the same prerequisite boundary as the CLI help. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "nix develop|port doctor|firecracker|PATH|machine launch" /home/alex/workspace/spoke-sh/port/README.md /home/alex/workspace/spoke-sh/port/docs/operators.md', proof: ac-2.log-->
<!-- verify: manual, SRS-03:start:end, proof: ac-3.log-->
- [ ] [SRS-03/AC-01] The published help-example workflow is recorded with direct CLI evidence in the documented environment. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo run -p port-cli -- doctor && nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo run -p port-cli -- --config examples/port.toml doctor', proof: ac-3.log-->

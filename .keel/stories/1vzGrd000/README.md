---
id: 1vzGrd000
title: Add PVM Doctor Host Kit Checks
type: feat
status: backlog
created_at: 2026-03-08T09:20:21
updated_at: 2026-03-08T09:21:51
scope: 1vz3ck000/1vzGo0000
---

# Add PVM Doctor Host Kit Checks

## Summary

Extend `port doctor` so the `x86_64/firecracker/pvm` lane reports explicit
host-kit readiness and blocking diagnostics instead of blurring unsupported
hosts into the standard Firecracker path.

## Acceptance Criteria

<!-- verify: manual, SRS-02:start:end, proof: ac-1.log, ac-2.log -->
- [ ] [SRS-02/AC-01] `port doctor` reports the x86_64 Firecracker/PVM host-kit check with explicit pass/fail diagnostics for platform, architecture, boot-line, and PVM Firecracker binary readiness. <!-- [SRS-02/AC-01] verify: cargo test -q -p port-runtime && cargo test -q -p port-cli, proof: ac-1.log -->
<!-- verify: manual, SRS-02:start:end, proof: ac-2.log -->
- [ ] [SRS-02/AC-02] Unsupported architecture, missing `pti=off`, and missing patched-binary states fail fast with clear messages and no fallback to the standard Firecracker lane. <!-- [SRS-02/AC-02] verify: cargo test -q -p port-runtime && cargo run -q -p port-cli -- --config examples/port.toml doctor, proof: ac-2.log -->

---
id: 1vzGrP000
title: Model X86 64 PVM Host Kit Contract
type: feat
status: in-progress
created_at: 2026-03-08T09:20:07
updated_at: 2026-03-08T09:22:45
scope: 1vz3ck000/1vzGo0000
started_at: 2026-03-08T09:22:45
---

# Model X86 64 PVM Host Kit Contract

## Summary

Define the explicit shared-model contract for the `x86_64/firecracker/pvm`
host kit and keep the `aarch64` PVM boundary research-only across model
rendering and example configuration.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end, proof: ac-1.log, ac-2.log -->
- [ ] [SRS-01/AC-01] `port-model` defines an implementation-ready `x86_64/firecracker/pvm` host-kit contract that captures the prepared-host boundary, boot-line expectation (`pti=off`), and patched Firecracker binary requirement. <!-- [SRS-01/AC-01] verify: cargo test -q -p port-model, proof: ac-1.log -->
<!-- verify: manual, SRS-01:start:end, proof: ac-2.log -->
- [ ] [SRS-01/AC-02] Example config and rendered model-facing output keep `x86_64` as the only planned PVM implementation lane and mark `aarch64/firecracker/pvm` as research-only with no silent compatibility claim. <!-- [SRS-01/AC-02] verify: cargo test -q -p port-model && cargo test -q -p port-cli, proof: ac-2.log -->

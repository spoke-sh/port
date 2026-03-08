---
id: 1vzHSA000
title: Select Local Pvm Runtime Inputs
type: feat
status: backlog
created_at: 2026-03-08T09:58:06
updated_at: 2026-03-08T09:59:34
scope: 1vz3ck000/1vzHPo000
---

# Select Local Pvm Runtime Inputs

## Summary

Teach the local Firecracker runtime path to select PVM-specific launch inputs
and fail with host-kit-specific diagnostics instead of treating PVM as a vague
future lane.

## Acceptance Criteria

<!-- verify: command, SRS-02:start:end, proof: ac-1.log, ac-2.log -->
- [ ] [SRS-02/AC-01] `port-runtime` resolves the PVM-specific Firecracker binary and launch metadata only when the requested machine selects `protection_mode = "pvm"`, while leaving the standard lane unchanged. <!-- [SRS-02/AC-01] verify: cargo test -q -p port-runtime, proof: ac-1.log -->
<!-- verify: command, SRS-02:start:end, proof: ac-2.log -->
- [ ] [SRS-02/AC-02] Local CLI proofs surface host-kit preflight failures as explicit PVM admission errors rather than falling back to the standard Firecracker lane. <!-- [SRS-02/AC-02] verify: cargo test -q -p port-cli && cargo run -q -p port-cli -- --config examples/port.toml doctor, proof: ac-2.log -->

---
id: 1vydip000
title: Deliver Guest Agent Capabilities
type: feat
status: in-progress
created_at: 2026-03-06T14:32:39
updated_at: 2026-03-06T15:03:35
scope: 1vydg7000/1vydgL000
started_at: 2026-03-06T15:03:35
---

# Deliver Guest Agent Capabilities

## Summary

Implement the guest agent transport and expose `exec`, `copy`, `pty`, `logs`,
and `forward` through the Port CLI and shared protocol.

## Acceptance Criteria

- [ ] [SRS-04/AC-01] The guest agent protocol supports request/response flows for `exec`, `copy`, `pty`, `logs`, and `forward`. <!-- [SRS-04/AC-01] verify: cargo test -p port-agent-protocol -p port-guest-agent, proof: ac-1.log-->
- [ ] [SRS-04/AC-02] The canonical CLI exposes `port guest exec`, `port guest copy`, `port guest pty`, `port guest logs`, and `port guest forward`. <!-- [SRS-04/AC-02] verify: bash -lc 'cargo run -p port-cli -- guest --help && cargo test -p port-cli --test guest_commands', proof: ac-2.log-->
- [ ] [SRS-04/AC-03] Automated tests cover protocol framing and at least one happy-path behavior for each guest capability. <!-- [SRS-04/AC-03] verify: cargo test, proof: ac-3.log-->

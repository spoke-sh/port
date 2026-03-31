---
# system-managed
id: VFTb0RCmf
status: done
created_at: 2026-03-31T12:50:29
updated_at: 2026-03-31T12:56:18
# authored
title: Make Guest Networking Default-On for Local Firecracker VMs
type: fix
operator-signal:
scope: VFTZdamHM/VFTarG3Jq
index: 1
started_at: 2026-03-31T12:51:34
submitted_at: 2026-03-31T12:56:18
completed_at: 2026-03-31T12:56:18
---

# Make Guest Networking Default-On for Local Firecracker VMs

## Summary

Add `Default` impl for `MachineNetworkSpec` with complete defaults (including DNS 8.8.8.8/8.8.4.4). In the Firecracker launch path, resolve `machine.network` from `Option` to effective config using `unwrap_or_default()` so networking activates by default even without explicit `[network]` config.

## Acceptance Criteria

- [x] [SRS-01/AC-01] When `MachineSpec.network` is `None`, the runtime uses `MachineNetworkSpec::default()` — Firecracker config includes `network-interfaces`, TAP + NAT are set up, network kernel cmdline params are passed. <!-- verify: manual, SRS-01:start:end -->
- [x] [SRS-NFR-01/AC-02] Explicit `[machines.*.network]` configs continue to work — serde defaults include DNS servers. <!-- verify: manual, SRS-NFR-01:start:end -->

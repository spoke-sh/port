# Default-On Networking Activation Fix - SRS

## Summary

Epic: VFTZdamHM
Goal: Make networking default-on by resolving None to MachineNetworkSpec::default() in the Firecracker launch path

## Scope

### In Scope

- [SCOPE-01] Add Default impl for MachineNetworkSpec with complete defaults including DNS servers and resolve machine.network to effective config in the Firecracker launch path.

### Out of Scope

- [SCOPE-02] Changes to the networking stack itself (TAP, NAT, guest init).

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | When `MachineSpec.network` is `None`, the Firecracker launch path must use `MachineNetworkSpec::default()` (enabled=true, 172.16.0.0/24, DNS 8.8.8.8/8.8.4.4). Explicit configs continue to work. | SCOPE-01 | FR-01 | cargo test + config inspection |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Explicit `[machines.*.network]` configs must continue to work unchanged. | SCOPE-01 | NFR-01 | cargo test |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Story Coverage Plan

| Story | Coverage |
|-------|----------|
| [VFTb0RCmf](../../../../stories/VFTb0RCmf/README.md) Make Guest Networking Default-On for Local Firecracker VMs | SRS-01, SRS-NFR-01 |

# Default-On Guest Networking Activation - Product Requirements

## Problem Statement

MachineSpec.network is Option with no Default — when TOML lacks [machines.demo.network], all networking code is skipped. Guest boots without eth0, default route, or egress. The dns_servers field also defaults to an empty vec, so even a bare [network] section gets no DNS.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Local Firecracker VMs get networking by default without requiring explicit config. | `port cluster up` with no `[network]` section produces a VM with working egress | `ping 8.8.8.8` succeeds inside the guest |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Port Operator | Operator running `port cluster up` for a local Firecracker-backed K3s cluster. | Guest VMs with working outbound networking by default. |

## Scope

### In Scope

- [SCOPE-01] Make networking default-on for local Firecracker VMs when no explicit config is provided.

### Out of Scope

- [SCOPE-02] Redesigning the networking stack or adding new network topologies.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | When `MachineSpec.network` is `None`, the runtime must use default networking config (enabled=true, 172.16.0.0/24 subnet, DNS 8.8.8.8/8.8.4.4) for local Firecracker VMs. | GOAL-01 | must | Without this, networking code is skipped entirely. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Explicit `[machines.*.network]` configs must continue to work unchanged. Operators can disable networking with `enabled = false`. | GOAL-01 | must | Backward compatibility. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Default activation | `cargo test` passes including config round-trip tests | Test output |
| Egress | `port cluster up` → guest can reach external hosts | Manual proof |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The structural networking code (TAP, NAT, guest init) from VFSRShGlI is correct. | Would need deeper fixes beyond activation. | Validate with rebuilt guest image. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Guest image must be rebuilt for init script changes to take effect. | Operator | Acknowledged |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `port cluster up` without `[machines.demo.network]` in TOML produces a Firecracker config with `network-interfaces`.
- [ ] Host-side TAP device and iptables rules are created.
- [ ] Guest boots with eth0 configured and a default route.
<!-- END SUCCESS_CRITERIA -->

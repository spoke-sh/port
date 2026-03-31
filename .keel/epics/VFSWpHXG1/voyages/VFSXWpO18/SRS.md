# TAP Networking and Host NAT for Local Firecracker VMs - SRS

> Deliver TAP/virtio-net guest networking with host-side NAT, guest DNS
> resolution, configurable port forwards, and the model/runtime changes
> to support them.

**Epic:** [VFSWpHXG1](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] TAP/virtio-net device creation and attachment to Firecracker VM configuration.
- [SCOPE-02] Host-side NAT (iptables MASQUERADE) for the guest subnet.
- [SCOPE-03] Guest-side DNS resolution configuration (`/etc/resolv.conf`).
- [SCOPE-04] Guest-side network interface bring-up (eth0 via DHCP or static config).
- [SCOPE-05] Kernel module availability for virtio-net and the network stack.
- [SCOPE-06] Network configuration fields in MachineSpec and FirecrackerConfig.
- [SCOPE-07] Operator-declared port forwards in port.toml or machine spec.

### Out of Scope

- [SCOPE-08] Bridged or routed networking topologies.
- [SCOPE-09] Changes to the existing vsock-based API forward mechanism.
- [SCOPE-10] Multi-node, AWS, or hosted-cluster networking.
- [SCOPE-11] Guest ingress from the internet — only outbound connectivity.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| The host kernel supports TAP devices and iptables MASQUERADE without additional setup. | dependency | Would need host-side kernel module or package prerequisites. |
| A single TAP + NAT topology is sufficient for the local single-node case. | assumption | Would need bridged or routed networking earlier than planned. |
| The Firecracker API accepts `network_interfaces` configuration for virtio-net. | dependency | Would require a different VMM network attachment strategy. |
| Guest kernel config includes virtio-net or can be rebuilt to include it. | dependency | Would need a kernel rebuild step before networking can work. |

## Constraints

- The vsock channel must remain the primary control plane — networking is for data-plane traffic only.
- Guest networking must be opt-in or default-on for local clusters, with the machine model supporting disable.
- NAT is sufficient; do not implement bridged or routed networking in this voyage.
- No changes to the existing API forward mechanism.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | FirecrackerConfig must include a `network_interfaces` field that attaches a host TAP device as the guest's virtio-net eth0. | SCOPE-01 | FR-01 | live VM boot + guest interface inspection |
| SRS-02 | Port must create a host-side TAP device before VM boot and configure iptables MASQUERADE for the guest subnet. | SCOPE-02 | FR-02 | host-side network inspection + guest outbound proof |
| SRS-03 | The guest image must ship a working `/etc/resolv.conf` pointing to reachable upstream nameservers. | SCOPE-03 | FR-03 | guest DNS resolution proof |
| SRS-04 | Guest init must bring up eth0 with an IP address (static or DHCP) before K3s starts. | SCOPE-04 | FR-04 | guest interface inspection on boot |
| SRS-05 | The guest kernel/initrd must include virtio-net and network stack modules. | SCOPE-05 | FR-05 | kernel module inspection |
| SRS-06 | MachineSpec must include network configuration fields to enable/disable guest networking and declare the guest subnet. | SCOPE-06 | FR-06 | model inspection + config toggle proof |
| SRS-07 | Operators must be able to declare additional host→guest port forwards beyond :6443 in port.toml or machine spec, and Port must establish them. | SCOPE-07 | FR-07 | host-side port reachability proof |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The vsock-based API forward must remain unchanged and functional. | SCOPE-01 | NFR-03 | existing API forward regression proof |
| SRS-NFR-02 | Guest networking must be disableable via the machine model for operators who do not want it. | SCOPE-06 | NFR-02 | config toggle proof |
| SRS-NFR-03 | The voyage must stay bounded to local single-node NAT networking; bridged, routed, AWS, and multi-node networking remain follow-on. | SCOPE-01 | NFR-01 | planning review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Story Coverage Plan

| Story | Coverage |
|-------|----------|
| [VFSXxsKe2](../../../../stories/VFSXxsKe2/README.md) Add TAP/virtio-net Interface and Host NAT to Firecracker VM Boot | SRS-01, SRS-02, SRS-06, SRS-NFR-01 |
| [VFSXyBMiG](../../../../stories/VFSXyBMiG/README.md) Configure Guest DNS Resolution and Network Interface Bring-Up | SRS-03, SRS-04, SRS-05, SRS-NFR-02 |
| [VFSXySLmb](../../../../stories/VFSXySLmb/README.md) Add Configurable Host-to-Guest Port Forwards | SRS-07, SRS-NFR-03 |

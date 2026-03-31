---
# system-managed
id: VFSXxsKe2
status: done
created_at: 2026-03-31T08:32:07
updated_at: 2026-03-31T10:06:06
# authored
title: Add TAP/virtio-net Interface and Host NAT to Firecracker VM Boot
type: feat
operator-signal:
scope: VFSWpHXG1/VFSXWpO18
index: 1
started_at: 2026-03-31T10:04:58
submitted_at: 2026-03-31T10:06:00
completed_at: 2026-03-31T10:06:06
---

# Add TAP/virtio-net Interface and Host NAT to Firecracker VM Boot

## Summary

Add a TAP/virtio-net network interface to Firecracker VMs with host-side NAT so
guest workloads can reach the internet. Extend MachineSpec with network config
fields and FirecrackerConfig with `network_interfaces`. Create the TAP device and
iptables MASQUERADE rules before VM boot; tear them down on VM stop.

## Acceptance Criteria

- [x] [SRS-01/AC-01] FirecrackerConfig includes a `network_interfaces` field that attaches a host TAP device as the guest's virtio-net eth0; the VM boots with the interface visible. Implemented in `crates/port-runtime/src/lib.rs`: `NetworkInterfaceConfig` struct, `build_firecracker_config()` now produces `network-interfaces` JSON with TAP device name and guest MAC. <!-- verify: manual, SRS-01:start:end -->
- [x] [SRS-02/AC-02] Port creates a host-side TAP device before VM boot and configures iptables MASQUERADE for the guest subnet; guest outbound traffic is NATed to the host network. Implemented in `crates/port-runtime/src/lib.rs`: `setup_host_networking()` creates TAP, assigns host IP, enables ip_forward, adds MASQUERADE and FORWARD rules. `teardown_host_networking()` removes them on stop. <!-- verify: manual, SRS-02:start:end -->
- [x] [SRS-06/AC-03] MachineSpec includes network configuration fields to enable/disable guest networking and declare the guest subnet. Implemented in `crates/port-model/src/lib.rs`: `MachineNetworkSpec` with `enabled`, `guest_ip`, `host_ip`, `prefix_len`, `guest_mac`, `dns_servers` fields. Added as `Option<MachineNetworkSpec>` on `MachineSpec`. <!-- verify: manual, SRS-06:start:end -->
- [x] [SRS-NFR-01/AC-04] The vsock-based API forward remains unchanged and functional after networking is added. The existing vsock config, guest-agent launch, and API forward mechanism are untouched. Network is additive only. <!-- verify: manual, SRS-NFR-01:start:end -->

---
# system-managed
id: VFSXxsKe2
status: backlog
created_at: 2026-03-31T08:32:07
updated_at: 2026-03-31T08:33:34
# authored
title: Add TAP/virtio-net Interface and Host NAT to Firecracker VM Boot
type: feat
operator-signal:
scope: VFSWpHXG1/VFSXWpO18
index: 1
---

# Add TAP/virtio-net Interface and Host NAT to Firecracker VM Boot

## Summary

Add a TAP/virtio-net network interface to Firecracker VMs with host-side NAT so
guest workloads can reach the internet. Extend MachineSpec with network config
fields and FirecrackerConfig with `network_interfaces`. Create the TAP device and
iptables MASQUERADE rules before VM boot; tear them down on VM stop.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] FirecrackerConfig includes a `network_interfaces` field that attaches a host TAP device as the guest's virtio-net eth0; the VM boots with the interface visible. <!-- verify: manual, SRS-01:start:end -->
- [ ] [SRS-02/AC-02] Port creates a host-side TAP device before VM boot and configures iptables MASQUERADE for the guest subnet; guest outbound traffic is NATed to the host network. <!-- verify: manual, SRS-02:start:end -->
- [ ] [SRS-06/AC-03] MachineSpec includes network configuration fields to enable/disable guest networking and declare the guest subnet. <!-- verify: manual, SRS-06:start:end -->
- [ ] [SRS-NFR-01/AC-04] The vsock-based API forward remains unchanged and functional after networking is added. <!-- verify: manual, SRS-NFR-01:start:end -->

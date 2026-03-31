---
# system-managed
id: VFSXyBMiG
status: done
created_at: 2026-03-31T08:32:09
updated_at: 2026-03-31T10:06:07
# authored
title: Configure Guest DNS Resolution and Network Interface Bring-Up
type: feat
operator-signal:
scope: VFSWpHXG1/VFSXWpO18
index: 2
started_at: 2026-03-31T10:04:58
submitted_at: 2026-03-31T10:06:00
completed_at: 2026-03-31T10:06:07
---

# Configure Guest DNS Resolution and Network Interface Bring-Up

## Summary

Configure the guest image so the VM has working DNS resolution and a fully
initialized network interface on boot. Ship `/etc/resolv.conf` with upstream
nameservers, ensure guest init brings up eth0 with a static IP, and verify the
guest kernel includes virtio-net and network stack modules.

## Acceptance Criteria

- [x] [SRS-03/AC-01] The guest image ships a working `/etc/resolv.conf` pointing to reachable upstream nameservers. Guest init parses `port.net_dns` from kernel cmdline and generates `/etc/resolv.conf` with comma-separated nameserver entries. <!-- verify: manual, SRS-03:start:end -->
- [x] [SRS-04/AC-02] Guest init brings up eth0 with a routable IP address before K3s starts. Guest init parses `port.net_ip`, `port.net_gateway`, `port.net_prefix_len` from kernel cmdline, waits for eth0, then configures IP and default route. <!-- verify: manual, SRS-04:start:end -->
- [x] [SRS-05/AC-03] The guest kernel/initrd includes virtio-net and network stack modules. Added `modprobe virtio_net` to initrd init script in `scripts/artifacts/build-guest-image.sh`. <!-- verify: manual, SRS-05:start:end -->
- [x] [SRS-NFR-02/AC-04] Guest networking can be disabled via the machine model. `MachineSpec.network` is `Option<MachineNetworkSpec>` — when absent or `enabled = false`, no TAP/NAT is created and no network cmdline args are passed, so the guest boots without networking. <!-- verify: manual, SRS-NFR-02:start:end -->

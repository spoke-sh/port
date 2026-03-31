---
# system-managed
id: VFSXyBMiG
status: backlog
created_at: 2026-03-31T08:32:09
updated_at: 2026-03-31T08:33:34
# authored
title: Configure Guest DNS Resolution and Network Interface Bring-Up
type: feat
operator-signal:
scope: VFSWpHXG1/VFSXWpO18
index: 2
---

# Configure Guest DNS Resolution and Network Interface Bring-Up

## Summary

Configure the guest image so the VM has working DNS resolution and a fully
initialized network interface on boot. Ship `/etc/resolv.conf` with upstream
nameservers, ensure guest init brings up eth0 with a static IP, and verify the
guest kernel includes virtio-net and network stack modules.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] The guest image ships a working `/etc/resolv.conf` pointing to reachable upstream nameservers. <!-- verify: manual, SRS-03:start:end -->
- [ ] [SRS-04/AC-02] Guest init brings up eth0 with a routable IP address before K3s starts. <!-- verify: manual, SRS-04:start:end -->
- [ ] [SRS-05/AC-03] The guest kernel/initrd includes virtio-net and network stack modules. <!-- verify: manual, SRS-05:start:end -->
- [ ] [SRS-NFR-02/AC-04] Guest networking can be disabled via the machine model. <!-- verify: manual, SRS-NFR-02:start:end -->

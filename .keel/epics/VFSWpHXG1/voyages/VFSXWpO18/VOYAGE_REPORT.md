# VOYAGE REPORT: TAP Networking and Host NAT for Local Firecracker VMs

## Voyage Metadata
- **ID:** VFSXWpO18
- **Epic:** VFSWpHXG1
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Add TAP/virtio-net Interface and Host NAT to Firecracker VM Boot
- **ID:** VFSXxsKe2
- **Status:** done

#### Summary
Add a TAP/virtio-net network interface to Firecracker VMs with host-side NAT so
guest workloads can reach the internet. Extend MachineSpec with network config
fields and FirecrackerConfig with `network_interfaces`. Create the TAP device and
iptables MASQUERADE rules before VM boot; tear them down on VM stop.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] FirecrackerConfig includes a `network_interfaces` field that attaches a host TAP device as the guest's virtio-net eth0; the VM boots with the interface visible. Implemented in `crates/port-runtime/src/lib.rs`: `NetworkInterfaceConfig` struct, `build_firecracker_config()` now produces `network-interfaces` JSON with TAP device name and guest MAC. <!-- verify: manual, SRS-01:start:end -->
- [x] [SRS-02/AC-02] Port creates a host-side TAP device before VM boot and configures iptables MASQUERADE for the guest subnet; guest outbound traffic is NATed to the host network. Implemented in `crates/port-runtime/src/lib.rs`: `setup_host_networking()` creates TAP, assigns host IP, enables ip_forward, adds MASQUERADE and FORWARD rules. `teardown_host_networking()` removes them on stop. <!-- verify: manual, SRS-02:start:end -->
- [x] [SRS-06/AC-03] MachineSpec includes network configuration fields to enable/disable guest networking and declare the guest subnet. Implemented in `crates/port-model/src/lib.rs`: `MachineNetworkSpec` with `enabled`, `guest_ip`, `host_ip`, `prefix_len`, `guest_mac`, `dns_servers` fields. Added as `Option<MachineNetworkSpec>` on `MachineSpec`. <!-- verify: manual, SRS-06:start:end -->
- [x] [SRS-NFR-01/AC-04] The vsock-based API forward remains unchanged and functional after networking is added. The existing vsock config, guest-agent launch, and API forward mechanism are untouched. Network is additive only. <!-- verify: manual, SRS-NFR-01:start:end -->

### Configure Guest DNS Resolution and Network Interface Bring-Up
- **ID:** VFSXyBMiG
- **Status:** done

#### Summary
Configure the guest image so the VM has working DNS resolution and a fully
initialized network interface on boot. Ship `/etc/resolv.conf` with upstream
nameservers, ensure guest init brings up eth0 with a static IP, and verify the
guest kernel includes virtio-net and network stack modules.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] The guest image ships a working `/etc/resolv.conf` pointing to reachable upstream nameservers. Guest init parses `port.net_dns` from kernel cmdline and generates `/etc/resolv.conf` with comma-separated nameserver entries. <!-- verify: manual, SRS-03:start:end -->
- [x] [SRS-04/AC-02] Guest init brings up eth0 with a routable IP address before K3s starts. Guest init parses `port.net_ip`, `port.net_gateway`, `port.net_prefix_len` from kernel cmdline, waits for eth0, then configures IP and default route. <!-- verify: manual, SRS-04:start:end -->
- [x] [SRS-05/AC-03] The guest kernel/initrd includes virtio-net and network stack modules. Added `modprobe virtio_net` to initrd init script in `scripts/artifacts/build-guest-image.sh`. <!-- verify: manual, SRS-05:start:end -->
- [x] [SRS-NFR-02/AC-04] Guest networking can be disabled via the machine model. `MachineSpec.network` is `Option<MachineNetworkSpec>` — when absent or `enabled = false`, no TAP/NAT is created and no network cmdline args are passed, so the guest boots without networking. <!-- verify: manual, SRS-NFR-02:start:end -->

### Add Configurable Host-to-Guest Port Forwards
- **ID:** VFSXySLmb
- **Status:** done

#### Summary
Allow operators to declare additional host→guest port forwards beyond the API
tunnel (:6443) in port.toml or machine spec. Port establishes these forwards
at boot so workstation tools can reach cluster services like MinIO console,
Envoy preview proxy, and Prometheus without manual kubectl port-forward.

#### Acceptance Criteria
- [x] [SRS-07/AC-01] Operators can declare additional host→guest port forwards in port.toml or machine spec, and Port establishes them at boot. `ServiceForwardSpec` added to `ClusterLifecycleSpec.forwards`; `cluster up` calls `ensure_detached_forward()` for each; `cluster down` tears them down. Example in `examples/port.toml`: `nodeport-http` and `nodeport-https`. <!-- verify: manual, SRS-07:start:end -->
- [x] [SRS-NFR-03/AC-02] The implementation stays bounded to local single-node NAT networking; no bridged, routed, AWS, or multi-node networking is introduced. All changes are scoped to local Firecracker TAP/NAT with static IPs on 172.16.0.0/24. <!-- verify: manual, SRS-NFR-03:start:end -->



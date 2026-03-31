# TAP Networking and Host NAT for Local Firecracker VMs - Software Design Description

> Deliver TAP/virtio-net guest networking with host-side NAT, guest DNS
> resolution, configurable port forwards, and the model/runtime changes
> to support them.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage adds the missing network data-plane to Port-managed Firecracker VMs.
Today the only host-guest channel is vsock (API forward on :6443 and guest-agent
control). Without a network interface the guest cannot resolve DNS, pull container
images, or clone Git repositories — blocking the entire Flux GitOps loop.

The design adds a TAP/virtio-net interface with host-side NAT in four coordinated
slices:

1. model and runtime: extend MachineSpec and FirecrackerConfig with network fields
2. host-side setup: create TAP device, assign IP, configure iptables MASQUERADE
3. guest-side setup: bring up eth0, configure DNS resolution
4. forward config: allow operator-declared port forwards beyond the API tunnel

## Context & Boundaries

### In Scope

- TAP device lifecycle (create before VM boot, teardown on VM stop)
- iptables MASQUERADE for a dedicated guest subnet
- Guest eth0 bring-up and `/etc/resolv.conf` configuration
- MachineSpec and FirecrackerConfig network fields
- Configurable host→guest port forwards

### Out of Scope

- Bridged or routed networking topologies
- Changes to the vsock-based API forward
- Multi-node, AWS, or hosted-cluster networking
- Inbound guest traffic from the internet

```
┌───────────────────────────────────────────────────────────────────┐
│                     Host (Port runtime)                           │
│                                                                   │
│  ┌──────────┐    ┌──────────┐    ┌────────────────────────┐      │
│  │ TAP setup│    │ iptables │    │ port-forward listeners │      │
│  │ + IP     │    │ MASQUERADE│    │ (operator-declared)    │      │
│  └────┬─────┘    └─────┬────┘    └───────────┬────────────┘      │
│       │                │                     │                    │
│  ─────┴────────────────┴─────────────────────┴──── tap0 ───────  │
└───────────────────────────────────────────────────────────────────┘
        │
┌───────┴───────────────────────────────────────────────────────────┐
│                 Firecracker VM (guest)                             │
│                                                                   │
│  eth0 (virtio-net) ── static IP ── /etc/resolv.conf               │
│       │                                                           │
│  K3s ─┤── CoreDNS (upstream resolution)                           │
│       ├── containerd (image pulls)                                │
│       └── Flux (Git clone)                                        │
│                                                                   │
│  vsock ── guest-agent ── API forward (:6443)  [unchanged]         │
└───────────────────────────────────────────────────────────────────┘
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Firecracker `network_interfaces` API | external VMM | attach virtio-net device to the VM | Firecracker REST API |
| Host kernel TAP support (`/dev/net/tun`) | host kernel | create the TAP device | Linux kernel |
| Host iptables/nftables | host tool | configure NAT MASQUERADE | system iptables |
| Guest kernel virtio-net module | guest kernel | drive the virtio-net device | guest kernel config |
| `port-model` MachineSpec | internal crate | express network policy | current workspace |
| `port-runtime` FirecrackerConfig | internal crate | configure the VM | current workspace |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Network topology | NAT via TAP + iptables MASQUERADE | Simplest topology for local single-node; bridged/routed is follow-on. |
| IP assignment | Static IP on a dedicated subnet (e.g., 172.16.0.0/24) | Avoids DHCP server complexity; predictable addressing for NAT rules. |
| DNS configuration | Static `/etc/resolv.conf` in the guest image pointing to host gateway or public resolvers | Avoids running a DNS forwarder; sufficient for the local case. |
| Control plane channel | vsock remains primary; TAP is data-plane only | Preserves the proven API forward path and avoids dual-channel complexity for control. |
| Network toggle | Enabled by default for local clusters, disableable in MachineSpec | Guest networking should just work for the common case but operators can opt out. |
| Forward config | Operator-declared in port.toml or MachineSpec, Port establishes socat/iptables DNAT | Extensible without code changes; keeps the API forward mechanism untouched. |

## Architecture

The voyage touches four cooperating layers:

1. **Model layer** (`port-model`): MachineSpec gains network config fields — enable/disable, guest subnet, and declared forwards.
2. **Runtime layer** (`port-runtime`): FirecrackerConfig gains `network_interfaces`; the boot sequence creates the TAP device and NAT rules before starting the VM.
3. **Guest layer** (guest image build): the image ships with virtio-net modules, static eth0 config, and working `/etc/resolv.conf`.
4. **Forward layer** (host-side): Port reads declared forwards from config and establishes host→guest port mappings at boot.

## Components

### Network Model Extension

- Purpose: express network policy and configuration in the machine model.
- Interface: new fields on MachineSpec — `networking.enabled`, `networking.guest_subnet`, `networking.forwards`.
- Behavior: defaults to enabled for local clusters; the runtime reads these fields to decide TAP/NAT setup.

### TAP Device and NAT Setup

- Purpose: create the host-side network path for guest traffic.
- Interface: called by the runtime boot sequence before Firecracker starts.
- Behavior: create TAP device, assign host-side IP (gateway), enable IP forwarding, add iptables MASQUERADE rule for the guest subnet. Teardown on VM stop.

### Firecracker Network Interface Config

- Purpose: attach the TAP device to the VM as a virtio-net interface.
- Interface: `network_interfaces` field in the Firecracker API PUT body.
- Behavior: reference the created TAP device by name, map to guest eth0.

### Guest Network Initialization

- Purpose: bring up eth0 inside the guest with a routable IP.
- Interface: guest init script or systemd unit.
- Behavior: `ip addr add` + `ip link set up` + `ip route add default` via the host gateway. Static config avoids DHCP dependency.

### Port Forward Establishment

- Purpose: expose operator-declared guest services on host ports.
- Interface: config entries in port.toml or MachineSpec `networking.forwards`.
- Behavior: for each declared forward, establish a host→guest mapping (socat or iptables DNAT) after the VM network is up.

## Interfaces

- Firecracker REST API: `PUT /network-interfaces/{iface_id}` with `host_dev_name` and `guest_mac`.
- Host TAP: `ip tuntap add dev <tap> mode tap` + `ip addr add` + `ip link set up`.
- Host NAT: `iptables -t nat -A POSTROUTING -s <guest_subnet> -o <host_iface> -j MASQUERADE`.
- Guest static config: `ip addr add <guest_ip>/24 dev eth0` + `ip route add default via <host_gw>`.

## Data Flow

1. Operator runs `port cluster up --cluster demo`.
2. Port reads MachineSpec; if `networking.enabled`, creates TAP device and NAT rules.
3. Port builds FirecrackerConfig with `network_interfaces` referencing the TAP device.
4. Firecracker boots the VM; guest init brings up eth0 with static IP.
5. Guest can now reach the internet via TAP → host NAT → host network.
6. CoreDNS resolves upstream, containerd pulls images, Flux clones repos.
7. Port reads declared forwards and establishes host→guest port mappings.
8. On `port cluster down`, Port tears down forwards, NAT rules, and TAP device.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| TAP device creation fails (permissions, kernel support) | `ip tuntap add` returns error | Fail VM boot with clear error message | Operator checks host kernel TAP support and permissions |
| iptables MASQUERADE fails | `iptables` command returns error | Fail VM boot; guest would have no outbound path | Operator checks iptables availability and permissions |
| Guest eth0 does not come up | Guest-agent health check or K3s DNS failures | Report networking degraded in cluster status | Check guest kernel modules and init script |
| DNS resolution fails in guest | CoreDNS upstream errors, Flux clone failures | Report DNS degraded in cluster status | Check `/etc/resolv.conf` and host gateway reachability |
| Port forward fails to bind | Host port already in use | Log warning, skip that forward | Operator changes the declared host port |

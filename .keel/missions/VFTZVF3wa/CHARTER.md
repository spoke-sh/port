# Complete Guest VM Outbound Networking - Charter

Archetype: Strategic

## Problem

The previous networking implementation (VFSRShGlI) added all the structural code — TAP setup, NAT rules, Firecracker `network_interfaces`, guest init eth0 bring-up — but gated it behind `Option<MachineNetworkSpec>`. When the TOML config lacks an explicit `[machines.demo.network]` section, `machine.network` is `None` and all networking code is skipped. The guest boots with only loopback, the K3s dummy `port0`, and `cni0` — no eth0, no default route, no egress. DNS is configured in `/etc/resolv.conf` but packets are unreachable.

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Networking activates by default for local Firecracker VMs — `port cluster up` produces a VM with working egress without requiring explicit `[network]` config. | board: VFTZdamHM |
| MG-02 | Guest `ip addr` shows eth0 with an IP, `ip route` shows a default route via the TAP gateway. | manual: `kubectl run --rm -it test --image=busybox -- nslookup github.com` succeeds |
| MG-03 | DNS defaults are complete — even a bare `[machines.demo.network]` section (or no section at all) gets working nameservers. | manual: CoreDNS resolves external hostnames through the full chain |

## Constraints

- Minimal change — fix the activation path, do not redesign the networking stack.
- Backward compatible — explicit `[machines.demo.network]` configs continue to work.
- Operators can still disable networking with `enabled = false`.

## Halting Rules

- DO NOT halt while local Firecracker VMs still boot without a default route.
- HALT when `port cluster up` with no `[network]` section produces a VM where `ping 8.8.8.8` succeeds.
- YIELD if the fix requires changes to the guest image build pipeline beyond what Port controls.

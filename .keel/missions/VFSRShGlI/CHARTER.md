# Enable Guest VM Outbound Networking - Charter

Archetype: Strategic

## Problem

Port-managed Firecracker VMs currently have no network interface. The only host-guest communication channel is vsock (used for the API forward on :6443 and guest-agent control). This means K3s CoreDNS cannot resolve external hostnames, Flux cannot clone GitRepository sources from GitHub, containerd cannot pull images from external registries, and any workload requiring outbound HTTP/HTTPS fails. This blocks the entire Flux GitOps reconciliation loop in spoke-sh/infra — the cluster is stuck at its bootstrap state with 5/18 components permanently degraded.

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Guest outbound networking: Attach a TAP/virtio-net interface to Firecracker VMs with host-side NAT so the guest can reach the internet. Flux can clone from GitHub, containerd can pull external images, CoreDNS can resolve. | board: VFSWpHXG1 |
| MG-02 | Guest DNS resolution: Configure `/etc/resolv.conf` in the guest image with working upstream nameservers (host-forwarded or public). CoreDNS upstream resolution works, K3s service DNS chain is healthy. | manual: `flux get sources git` shows all GitRepositories as `Ready: True` |
| MG-03 | Configurable service forwards: Allow operators to declare additional host→guest port forwards beyond the API (6443) so workstation tools can reach cluster services like MinIO console, Envoy preview proxy, and Prometheus. | manual: `infra health --env local --scene` shows 18/18 OPERATIONAL |

## What Needs to Change in Port

| Area | Current State | Required Change |
|------|---------------|-----------------|
| FirecrackerConfig struct (`port-runtime/src/lib.rs`) | No `network_interfaces` field | Add network interface config (TAP device → guest eth0) |
| MachineSpec model (`port-model/src/lib.rs`) | No network configuration fields | Add network policy/config to the machine model |
| Guest image build (`scripts/artifacts/build-guest-image.sh`) | No `/etc/resolv.conf`, no network interface setup | Add DNS config, ensure guest init brings up eth0 via DHCP or static config |
| Host-side setup | No TAP device creation, no NAT rules | Create TAP device, configure iptables MASQUERADE for guest subnet |
| Guest kernel/initrd | Loads `virtio_mmio` but no net modules explicitly | Ensure virtio-net and network stack modules are available |
| Forward config | Single hardcoded API forward | Allow operator-declared forwards in `port.toml` or machine spec |

## Constraints

- The vsock channel must remain the primary control plane — networking is for data-plane traffic only.
- Guest networking should be opt-in or default-on for local clusters, but the machine model must support disabling it.
- NAT is sufficient for the local single-node case. Bridged or routed networking can remain follow-on.
- No changes to the existing API forward mechanism — it works and should stay as-is.

## Halting Rules

- DO NOT halt while Flux GitOps reconciliation remains blocked on missing guest outbound networking or DNS resolution.
- HALT when `kubectl exec` into a pod can `wget https://github.com`, `flux get sources git` shows all GitRepositories as `Ready: True`, and `infra health --env local --scene` shows 18/18 OPERATIONAL.
- YIELD to human when the remaining blocker requires a product decision on network topology (bridged vs routed), security policy for guest egress, or non-local scope expansion rather than implementation work.

# Guest VM Outbound Networking - Product Requirements

## Problem Statement

Firecracker VMs have no network interface — only vsock exists. This blocks CoreDNS resolution, Flux GitRepository cloning, containerd image pulls, and all outbound HTTP/HTTPS workloads, leaving the Flux GitOps loop in spoke-sh/infra stuck at 5/18 components.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Attach a TAP/virtio-net interface to Firecracker VMs with host-side NAT so the guest can reach the internet. | Pod inside the guest VM can resolve DNS and reach external HTTPS endpoints | `kubectl exec` into a pod and `wget https://github.com` succeeds |
| GOAL-02 | Configure working DNS resolution inside the guest image. | CoreDNS upstream resolution works, K3s service DNS chain is healthy | `flux get sources git` shows all GitRepositories as `Ready: True` |
| GOAL-03 | Allow operators to declare additional host→guest port forwards beyond the API (6443). | Workstation tools can reach cluster services without manual `kubectl port-forward` | Declared forwards in `port.toml` or machine spec are reachable from the host |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Port Operator | The operator using `port cluster up` to run a local Firecracker-backed K3s cluster. | Guest VMs that can reach the internet so Flux, containerd, and CoreDNS work. |

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

- [SCOPE-08] Bridged or routed networking topologies — NAT is sufficient for local single-node.
- [SCOPE-09] Changes to the existing vsock-based API forward mechanism.
- [SCOPE-10] Multi-node, AWS, or hosted-cluster networking.
- [SCOPE-11] Guest ingress from the internet — only outbound connectivity is in scope.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Firecracker VM configuration must include a `network_interfaces` field that attaches a TAP device as the guest's virtio-net eth0. | GOAL-01 | must | Without a network interface the guest has no path to the network. |
| FR-02 | Port must create and configure a host-side TAP device before VM boot and set up iptables MASQUERADE for the guest subnet. | GOAL-01 | must | The TAP device is the host-side anchor for guest networking; NAT provides outbound connectivity. |
| FR-03 | The guest image must include a working `/etc/resolv.conf` pointing to reachable upstream nameservers. | GOAL-02 | must | DNS resolution is required for CoreDNS, Flux, and containerd. |
| FR-04 | The guest init must bring up eth0 with an IP address (static or DHCP) on boot. | GOAL-01 | must | The interface must be configured before K3s starts. |
| FR-05 | The guest kernel/initrd must include virtio-net and network stack modules. | GOAL-01 | must | Without the kernel modules the interface cannot be used. |
| FR-06 | MachineSpec must include network configuration fields to enable/disable guest networking. | GOAL-01 | must | The machine model must express network policy so operators can control it. |
| FR-07 | Operators must be able to declare additional host→guest port forwards beyond the API (6443) in port.toml or machine spec. | GOAL-03 | must | Cluster services like MinIO, Envoy preview, and Prometheus need to be reachable from the host. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The vsock channel must remain the primary control plane — networking is for data-plane traffic only. | GOAL-01 | must | Preserves the existing stable control path. |
| NFR-02 | Guest networking must be opt-in or default-on for local clusters, with the machine model supporting disable. | GOAL-01 | should | Operators need control over network exposure. |
| NFR-03 | No changes to the existing API forward mechanism. | GOAL-01, GOAL-03 | must | The vsock-based API forward works and must remain untouched. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Guest outbound networking | `kubectl exec` into a pod and `wget https://github.com` | Story-level proof artifact |
| DNS resolution | `flux get sources git` shows all GitRepositories as `Ready: True` | Story-level proof artifact |
| Full GitOps reconciliation | `infra health --env local --scene` shows 18/18 OPERATIONAL | Story-level proof artifact |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The host kernel supports TAP devices and iptables MASQUERADE without additional setup. | Would need host-side kernel module or package prerequisites. | Validate during first execution slice. |
| A single TAP + NAT topology is sufficient for the local single-node case. | Would need bridged or routed networking earlier than planned. | Validate with the full Flux reconciliation loop. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which guest subnet CIDR to use and whether it conflicts with K3s pod/service CIDRs. | Epic owner | Open |
| Whether guest image init should use DHCP or static IP configuration. | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `kubectl exec` into a pod and `wget https://github.com` succeeds.
- [ ] `flux get sources git` shows all GitRepositories as `Ready: True`.
- [ ] `infra health --env local --scene` shows 18/18 OPERATIONAL.
- [ ] Guest networking can be disabled via the machine model.
- [ ] No changes to the vsock-based API forward mechanism.
<!-- END SUCCESS_CRITERIA -->

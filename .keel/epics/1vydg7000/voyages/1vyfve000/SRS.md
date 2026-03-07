# Wire Live Guest Transport - Software Requirements Specification

> Make the canonical port guest flows work against launched Firecracker VMs through a live guest transport

**Epic:** [1vydg7000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

- [SCOPE-01] Stabilize local runtime state and guest-operation diagnostics so a
  launched machine can be relaunched cleanly and transport failures are
  actionable.
- [SCOPE-02] Connect the canonical `port guest` flows for `exec`, `pty`, and
  `logs` to a launched Firecracker VM through a live guest transport.
- [SCOPE-03] Rework `copy` and `forward` so they remain coherent across a real
  host/guest boundary instead of relying on shared host paths or guest-local
  listeners.
- [SCOPE-04] Update help and operator docs so the CLI accurately describes the
  live guest transport behavior and current forward lifecycle.
- Out of scope: remote cloud launch orchestration, Firecracker API enablement,
  and new non-Linux runtime support.

## Assumptions & Dependencies

<!-- What we assume to be true; external systems, services, or conditions we depend on -->

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Firecracker continues to expose the guest vsock tunnel through the host-side Unix socket in the runtime directory. | Dependency | The live transport design would need a different host/guest bridge. |
| The guest image can start `port-guest-agent` early in boot with both filesystem and vsock access. | Dependency | Live guest operations would still not be reachable after launch. |
| The Linux MVP lane remains the only execution target for end-to-end live-transport validation. | Assumption | Verification scope would need to expand into unsupported operator lanes. |

## Constraints

- The canonical operator surface remains the `port` CLI; hidden manual
  Firecracker steps are not an acceptable substitute for guest operations.
- `port machine launch` continues to run Firecracker with `--no-api`, so the
  live transport must not depend on the Firecracker REST API.
- The solution must preserve a usable local test path without requiring a real
  VM for every unit/integration test.
- Forwarding semantics must be honest about lifecycle: the CLI cannot claim a
  durable background host listener unless Port is actually keeping one alive.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Local launch must clean stale runtime state from prior Firecracker runs and fail fast with an actionable message when the requested machine is already running. | SCOPE-01 | FR-02 | automated test + CLI proof |
| SRS-02 | A launched guest image must expose `port-guest-agent` on the machine’s configured guest control port so the runtime can reach it through Firecracker vsock. | SCOPE-02 | FR-03 | automated test + launched-VM proof |
| SRS-03 | `port guest exec`, `port guest pty`, and `port guest logs` must work against a launched Firecracker VM through the canonical CLI and shared machine model. | SCOPE-02 | FR-01 | automated test + launched-VM proof |
| SRS-04 | `port guest copy` must transfer bytes across the real host/guest boundary in both directions without assuming the guest agent can see host filesystem paths directly. | SCOPE-03 | FR-04 | automated test + launched-VM proof |
| SRS-05 | `port guest forward` must provide a coherent host-side workflow that proxies traffic to a guest target through the guest transport and surfaces its lifecycle honestly in the CLI/docs. | SCOPE-03, SCOPE-04 | FR-05 | automated test + launched-VM proof |
| SRS-06 | Help text and operator docs must describe the live guest transport behavior, including the current forward lifecycle and any remaining constraints. | SCOPE-04 | FR-06 | doc review + CLI proof |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Transport selection and failure handling must remain deterministic: runtime socket when explicitly present, live Firecracker transport when the VM is launched, otherwise an actionable error. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-01 | automated test + inspection |
| SRS-NFR-02 | The voyage must add automated coverage for the new transport/protocol behavior and record CLI-level live-VM evidence for the canonical guest workflows. | SCOPE-02, SCOPE-03, SCOPE-04 | NFR-02 | automated test + CLI proof |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

# Architecture

Port is organized around one shared model and one operator vocabulary that can
route either to a local runtime owner or to a hosted control plane plus node
agent.

## Core Shape

```text
port CLI / port-sdk
        |
        v
   shared Port model
        |
        +-------------------------+
        |                         |
        v                         v
 local runtime owner       hosted control plane
        |                         |
        v                         v
 artifact store,            node agent, hosted
 runtime roots,             runtime roots, placement,
 guest transport            service supervision
        \                         /
         \                       /
          +------ guest agent ---+
```

## Current Production Posture

Port's execution hierarchy is intentionally uneven today:

| Posture | Why it exists |
|--------|----------------|
| Local Firecracker `standard` | Default Linux lane for direct proof and operator workflow development |
| Hosted Firecracker `standard` | Proves the live hosted control-plane/node-agent split and keeps guest/service verbs canonical |
| Hosted AWS `x86_64` Firecracker/PVM | Strongest production-oriented cloud path because it carries the prepared-host, artifact-kit, and no-fallback contract |

If you are evaluating Port for a production-shaped AWS rollout, start with
[`docs/aws.md`](docs/aws.md). This file explains the component boundaries; the
AWS guide explains how those boundaries come together operationally.

## Major Boundaries

### CLI And SDK

- `crates/port-cli` exposes the operator surface.
- `crates/port-sdk` mirrors the hosted API as a typed client surface.
- Both depend on the shared model instead of owning separate workflow logic.

### Shared Model

- `crates/port-model` defines artifacts, hosts, nodes, control planes,
  machines, service policy, and lane selectors.
- `examples/port.toml` is the checked-in sample expression of that model.

### Runtime Layer

- `crates/port-runtime` owns artifact resolution, doctor checks, launch
  orchestration, hosted routing, and service supervision.
- The runtime layer decides whether a request stays local or routes through the
  hosted control-plane split.

### Hosted Control Plane

- Resolves machine placement and route ownership.
- Persists hosted state under `.port/hosted/<control-plane>/...`.
- Exposes the canonical hosted HTTP routes used by both the CLI and the SDK.

### Node Agent

- Owns node-local runtime state for hosted machines and services.
- Serves hosted machine, guest, and service operations behind the control
  plane.

### Guest Protocol And Guest Agent

- Guest operations use the shared `port-agent-protocol`.
- Local Firecracker, hosted control-plane, Cloud Hypervisor, and AVF lanes
  reuse the same guest command family.

## Execution Lanes

| Lane | Current role |
|------|--------------|
| Firecracker `standard` | Default local Linux lane |
| Firecracker `pvm` on `x86_64` | Prepared-node hosted AWS lane and strongest production-oriented cloud contract |
| Cloud Hypervisor `standard` | Proof-backed local and hosted lane |
| AVF `standard` | Proof-backed local macOS lane |

## Cluster Contracts

Port now has two different K3s cluster contracts under the same `port cluster`
verbs:

- local K3s: one local Firecracker `standard` microVM is the cluster node
- hosted K3s: one or more hosted Firecracker guest microVMs are the cluster
  nodes, launched through the hosted control plane and node agents

The execution host and the K3s node are not the same layer in the hosted
contract:

- the AWS PVM host is an execution host that runs Port node-agent ownership
- the K3s control-plane and worker nodes are guest microVMs launched on top of
  those execution hosts

Real HA is therefore stricter than "multiple nodes":

- at least three control-plane microVMs
- a stable HTTPS API endpoint fronting them
- control-plane microVMs placed across distinct execution hosts so one host
  loss does not remove quorum

Port models the K3s topology and endpoint contract, but it does not ship the
external load balancer, VIP, DNS, or ingress layer that fronts a real HA
control plane.

## Artifact System

Artifacts are modeled as logical references plus concrete variants.

- A variant is selected by `architecture`, `substrate`, and `protection_mode`.
- Distribution backends are explicit: file-system, hosted-api, and
  `oci-registry`.
- Port resolves one canonical variant at a time for build, validate, push, and
  pull.

## Service System

`port service` is the canonical surface for:

- secret management
- long-lived services
- sandbox execution
- restart and health policy

The service runtime keeps one policy model across local and hosted ownership.

## Documentation Contract

Top-level docs are intentionally split by role:

- root docs define stable contracts such as configuration, architecture,
  release, and evaluations
- focused docs cover a specific lane or subsystem, with
  [`docs/aws.md`](docs/aws.md) now acting as the canonical deployment narrative
  for AWS hosted PVM
- `port --help` and the README stay concise and link outward instead of
  duplicating long workflows

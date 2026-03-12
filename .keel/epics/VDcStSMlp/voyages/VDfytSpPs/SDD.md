# Hosted Stateless K3s Foundations - Software Design Description

> Deliver one hosted-control-plane K3s workflow with a fixed one-server-plus-
> worker topology, canonical cluster access, explicit hosted-only boundaries,
> and a proof-backed operator path.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage turns the K3s epic into one bounded delivery slice. It does not try
to ship a general Kubernetes platform. Instead, it adds one explicit hosted
K3s contract, reuses the existing hosted control-plane and guest-attach path to
bootstrap a fixed cluster topology, exposes cluster access through canonical
surfaces, and publishes one reviewable proof artifact.

## Context & Boundaries

### In Scope

- hosted K3s cluster contract and validation
- one fixed hosted topology: one server node plus at least one worker node
- bootstrap and join through canonical machine and guest surfaces
- cluster access handoff and explicit hosted-only boundary guidance
- proof-backed docs for the first hosted K3s workflow

### Out of Scope

- HA, multi-server control planes, or autoscaling clusters
- hosted persistent storage, CSI, or stateful workload claims
- ingress, public exposure, or load-balancer productization
- SSH-first or multi-provider cluster orchestration
- a second Kubernetes-only operator toolchain

```
┌──────────────────────────────────────────────────────────────────┐
│              Hosted Stateless K3s Foundations                   │
│                                                                  │
│  cluster contract + validation ────┐                             │
│                                     ├──> hosted bootstrap/join    │
│  hosted route + guest surfaces ─────┤      (server + worker)      │
│                                     │                             │
│  access + proof artifacts ──────────┘                             │
└──────────────────────────────────────────────────────────────────┘
          ↑                            ↑
    hosted control plane          one host group
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `crates/port-model` hosted inventory, host-group, machine, and guest contracts | internal code | anchor the K3s contract to existing control-plane, host-group, and machine vocabulary | current workspace |
| `crates/port-runtime::hosted_control_plane` | internal code | reuse hosted placement, selected-node resolution, and route context for cluster bootstrap and lifecycle | current workspace |
| `crates/port-hosted-protocol` hosted route context and transport vocabulary | internal code | preserve explicit ownership and route detail in K3s lifecycle surfaces | current workspace |
| existing hosted machine and guest CLI surfaces | internal code | bootstrap and inspect K3s without inventing a second remote toolchain | current workspace |
| proof system with recording-backed artifacts | board workflow | capture a human-reviewable hosted K3s workflow for story and mission review | current Keel verification toolchain |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| First topology | one hosted K3s server plus one hosted worker in one host group | smallest slice that still proves cluster bootstrap and worker join |
| Ownership lane | hosted control plane only | hosted routing and host-group placement are the strongest current substrate; SSH and multi-provider parity can follow later |
| Cluster contract | introduce an explicit K3s-specific config contract instead of pretending Port already needs a generic Kubernetes abstraction | keeps the first slice honest and avoids premature distro-generalization |
| Execution surface | reuse canonical `port machine` and `port guest` operations underneath the K3s workflow | preserves product coherence and avoids a second operator toolchain |
| Storage stance | stateless only | hosted attached-volume routing is not ready, so persistence must stay explicit as out of scope |
| Human proof | use a recording-backed artifact through the proof system, likely via a repo-local renderer script that emits cast and gif artifacts | matches the repo's current reliable proof pattern when direct `vhs` capture is unstable in SSH or tmux shells |

## Architecture

The voyage introduces four coordinated layers:

1. hosted K3s cluster contract and validation
2. hosted bootstrap and worker-join orchestration
3. cluster access and boundary surfaces
4. docs and proof artifacts

## Components

### Hosted K3s Cluster Contract

- Purpose: define one explicit K3s workload lane without replacing the existing
  machine and guest model.
- Interface: a K3s-specific config block that references one control plane, one
  host group, one server machine, one or more worker machines, and minimal
  bootstrap metadata such as K3s version or install arguments.
- Behavior: validate that referenced machines, host group, and ownership route
  are compatible with the hosted-first K3s slice.

### Hosted Bootstrap Coordinator

- Purpose: realize the fixed K3s topology using current hosted placement and
  guest-control primitives.
- Interface: canonical machine lifecycle plus guest-control operations, wrapped
  by the K3s workflow implementation.
- Behavior: launch or verify the server machine, install or bootstrap K3s,
  extract the join token or equivalent metadata, then join worker nodes
  through the same hosted route.

### Cluster Access And Boundary Surface

- Purpose: make the resulting cluster usable and its current limitations
  explicit.
- Interface: cluster status output, kubeconfig or equivalent access handoff,
  route-aware failure messages, and docs.
- Behavior: surface control-plane, host-group, selected-node, rejected-node,
  and unsupported-lane detail instead of hiding cluster ownership behind
  generic Kubernetes language.

### Hosted K3s Proof Surface

- Purpose: publish a human-reviewable workflow for cluster bring-up and review.
- Interface: docs plus a recording-backed proof artifact.
- Behavior: show one coherent hosted K3s workflow from bootstrap through
  cluster-access or workload visibility without implying persistence or ingress
  support.

## Interfaces

- a new K3s-specific config contract, likely under a top-level cluster catalog
  such as `[k3s_clusters.<name>]`
- references to existing `machines.<name>`, `host_groups.<name>`, and hosted
  `control_planes.<name>` entries
- canonical underlying commands:
  - `port machine launch --machine <name>`
  - `port machine status --machine <name>`
  - `port guest exec --machine <name> -- <command>`
  - `port guest copy --machine <name> ...`
- proof command, likely via a repo-local renderer script that records cast and
  gif artifacts into story evidence

## Data Flow

1. Operator defines a hosted K3s cluster that references one control plane, one
   host group, one server machine, and one or more worker machines.
2. Config validation checks hosted ownership, host-group membership, machine
   references, and the stateless first-slice boundary.
3. The K3s workflow launches the server machine through the hosted control
   plane and bootstraps K3s on that server through canonical guest-control
   surfaces.
4. The workflow captures join metadata and uses the same hosted route to launch
   and join worker machines.
5. Cluster access is handed back through kubeconfig or equivalent cluster
   access output, with node or workload visibility proving the cluster is
   usable.
6. Docs and the proof artifact render that same workflow for human review.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Cluster references a host group or machines that are not compatible with the hosted K3s lane | config validation or hosted placement resolution | fail fast with explicit control-plane, host-group, machine, and route detail | correct the cluster contract or hosted inventory, then retry |
| Hosted placement cannot find a live node for the server or worker machines | hosted route resolution | surface candidate-node, rejected-node, and placement-detail context | restore node registration or choose compatible machines and host group |
| Workflow requests persistence, HA, ingress, or an SSH-owned route | validation or preflight | reject the request with explicit first-slice boundary guidance | keep the workflow inside the hosted stateless contract or move the request to a later epic |
| Cluster boots but access handoff cannot produce usable node or workload visibility | automated test or command proof | fail the story and keep cluster access as an explicit requirement gap | restore kubeconfig or equivalent access output before submission |
| Proof artifact drifts from the actual hosted workflow | proof review or story verification | regenerate the recording from the canonical command path | keep proof commands tied to story verification |

## Story Decomposition

1. Contract story: add the canonical hosted K3s cluster contract and validation.
2. Bootstrap story: implement hosted server bootstrap and worker join.
3. Access and boundary story: surface cluster access, placement detail, and
   explicit hosted-only failure boundaries.
4. Proof story: publish docs and a recording-backed hosted K3s workflow.

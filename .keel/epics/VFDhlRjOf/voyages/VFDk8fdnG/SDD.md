# Plan Single-Node Local Cluster Surface - Software Design Description

> Define the first cluster-first local K3s workflow with Port-owned bootstrap
> inputs, direct cluster lifecycle and kubeconfig surfaces, and an explicit
> infra handoff.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage takes the K3s primitives Port already ships and raises them to one
cluster-oriented local workflow. It does not try to solve multi-node
networking, hosted or AWS placement, or a generic Kubernetes platform. Instead,
it introduces a new cluster-facing operator surface for one named local K3s
cluster, feeds that workflow with Port-owned bootstrap inputs, and makes Port
the owner of cluster health and kubeconfig handoff.

## Context & Boundaries

### In Scope

- one named local K3s cluster contract and cluster-facing CLI surface
- one single-node local execution path
- Port-owned offline bootstrap inputs and kube-ready guest-profile preparation
- direct cluster health and kubeconfig handoff
- docs and proof for the thin infra seam

### Out of Scope

- multi-node local or hosted cluster orchestration
- AWS, cross-node networking, CIDR allocation, or stable inter-node addressing
- ingress, public endpoints, or broader Kubernetes platform semantics
- persistent volumes, CSI, or stateful workload claims
- downstream infra bootstrap implementation beyond a proof-backed Port handoff

```
┌─────────────────────────────────────────────────────────────────┐
│            Single-Node Local Cluster Operator Surface          │
│                                                                 │
│  cluster command + named contract ─────┐                        │
│                                         ├──> local cluster       │
│  offline K3s kit + guest profile ──────┤      coordinator        │
│                                         │                        │
│  health + kubeconfig output ────────────┘                        │
│                 │                                                │
│                 └────────> docs + infra handoff proof            │
└─────────────────────────────────────────────────────────────────┘
             ↑                          ↑
       local machine lane         downstream infra
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `crates/port-model` machine, artifact, host, and existing K3s contract vocabulary | internal code | anchor the first cluster resource to Port's existing model rather than inventing a second system | current workspace |
| `crates/port-runtime` local machine launch and guest-operation surfaces | internal code | reuse local runtime and guest-control paths under the cluster coordinator | current workspace |
| existing hosted-K3s bootstrap and access helpers | internal code | seed install, kubeconfig, and health semantics while the operator surface changes | current workspace |
| cluster-proof docs and Keel evidence surfaces | board workflow | publish the new operator contract and a human-reviewable proof path | current workspace |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| First supported lane | single-node local K3s only | current guest networking gaps make multi-node or hosted expansion a poor first deployment target |
| Operator vocabulary | add a cluster-facing Port surface for named clusters | infra should ask Port for a cluster outcome, not orchestrate raw VM primitives |
| Backing model | reuse or evolve Port's current K3s configuration model under a cluster-first surface | preserves product continuity while allowing a simpler external contract |
| Bootstrap ownership | Port-owned artifact staging or kube-ready guest profile; no guest-side `curl get.k3s.io` | removes the current brittle live-fetch path and keeps installation inputs explicit |
| Health ownership | Port owns cluster readiness and kubeconfig handoff; downstream infra owns later GitOps convergence | separates Port's product contract from downstream bootstrap behavior |
| Follow-on boundary | multi-node, AWS, hosted expansion, and richer networking become later work | keeps this voyage honest and shippable |

## Architecture

The voyage introduces five coordinated layers:

1. cluster catalog and CLI surface
2. local bootstrap kit and guest-profile preparation
3. cluster lifecycle coordinator
4. cluster health and kubeconfig surfaces
5. docs and infra handoff proof

## Components

### Cluster Catalog And CLI Surface

- Purpose: give operators one named cluster resource instead of a raw sequence
  of machine and guest commands.
- Interface: a new cluster-oriented CLI family for named clusters plus a config
  contract that binds the first cluster lane to local K3s.
- Behavior: validate single-node local constraints and route cluster actions to
  the coordinator without exposing raw join-token or API-forward steps.

### Local Bootstrap Kit And Guest Profile

- Purpose: replace guest-side live fetches with Port-owned installation inputs.
- Interface: artifact selection, guest copy or staging paths, and a guest image
  or profile that already carries the dependencies needed for the first K3s
  install.
- Behavior: materialize versioned K3s inputs locally, stage them into the
  guest, and make bootstrap independent of `curl https://get.k3s.io`.

### Cluster Lifecycle Coordinator

- Purpose: own `up`, `status`, and `down` behavior for the first local cluster.
- Interface: cluster-facing commands backed by existing local machine launch and
  guest-control primitives.
- Behavior: ensure the cluster machine is launched, install or start K3s from
  staged inputs, evaluate readiness, and stop the cluster cleanly.

### Cluster Health And Kubeconfig Surface

- Purpose: make Port the owner of cluster readiness and access handoff.
- Interface: `status` and `kubeconfig` style surfaces, plus route-aware failure
  messages and boundary guidance.
- Behavior: report whether the cluster is healthy, return a directly usable
  kubeconfig, and avoid external kubeconfig rewriting or detached forwarding
  choreography in the canonical path.

### Operator Contract And Infra Handoff Proof

- Purpose: publish the intended seam between Port and downstream infra.
- Interface: CLI help, docs, and one recording-backed proof artifact.
- Behavior: show that the infra-facing contract is now "obtain healthy cluster
  plus kubeconfig from Port", while keeping multi-node and hosted expansion
  explicit follow-on scope.

## Interfaces

- cluster-facing CLI, likely under `port cluster ...`
- named cluster config contract for the first local K3s lane
- existing underlying machine and guest operations used as implementation
  substrate, not as the blessed operator workflow
- proof commands and docs that render the local cluster path for review

## Data Flow

1. Operator defines or selects a named local cluster.
2. Port validates that the request stays inside the first-slice boundary:
   local lane, single-node, supported K3s flavor, no multi-node expansion.
3. Port resolves the local machine and the offline bootstrap kit or guest
   profile needed to install K3s.
4. The cluster coordinator launches the local machine, stages the install
   inputs, and bootstraps K3s without guest-side live fetches.
5. Port evaluates cluster readiness and returns health plus a usable kubeconfig
   directly from the cluster surface.
6. Docs and proof artifacts publish that same flow as the thin handoff contract
   for downstream infra.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Request targets multi-node, hosted, or AWS cluster behavior | config validation or cluster command preflight | fail fast with explicit first-slice boundary guidance | keep the request on single-node local or defer it to a follow-on mission |
| Offline bootstrap kit or guest profile is missing required K3s inputs | cluster `up` preflight or staging step | fail before guest install begins and report the missing Port-owned input | build or stage the required asset, then rerun |
| Local machine launches but K3s does not become healthy | readiness check or status surface | report cluster-unhealthy state through the cluster surface instead of forcing infra to infer it indirectly | inspect logs or fix bootstrap inputs, then retry |
| Kubeconfig is empty or requires manual rewriting | `kubeconfig` surface or proof path | fail the story and keep kubeconfig handoff as a product gap | return a directly usable kubeconfig before submission |
| Docs or proof still show raw guest choreography as the blessed path | doc inspection or proof review | reject the story and update the operator contract | re-render docs/help/proof from the canonical cluster workflow |

## Story Decomposition

1. Contract story: add the cluster-facing CLI and named local cluster contract.
2. Bootstrap story: stage offline K3s inputs and the kube-ready guest profile.
3. Lifecycle story: implement cluster `up`, `status`, `kubeconfig`, and `down`
   with explicit health ownership.
4. Proof story: publish docs, help, and the infra handoff proof path.

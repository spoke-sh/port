# K3s And Kubernetes Workloads — Brief

## Hypothesis

Port can now grow into a higher-level workload platform if it ships one narrow
first-class K3s lane built on the verified installable CLI, hosted placement
and service primitives, and proof workflows instead of treating Kubernetes as
an unrelated external add-on.

## Problem Space

Port now has installable Linux and macOS packages, hosted node and host-group
placement, hosted service workflows, SSH-managed remote lifecycle, and a first
attached-volume contract, but the clearest higher-level workload outcome is
still missing. The question is no longer whether Port can ever support K3s; it
is what the smallest canonical first slice should be without overcommitting to
HA clusters, persistent storage, or a generic Kubernetes platform promise.

## Context

Earlier research parked K3s because developer experience, hybrid execution, and
storage foundations were still unsettled. Those foundations are now verified,
while the hosted docs and proof scripts make host-group placement, selected
node identity, and service lifecycle operator-visible through the canonical
`port` CLI.

Slicer's public examples still matter because they show what humans recognize
as a platform capability: a real cluster outcome, not just another fleet or VM
primitive. The reassessment problem is to make Port's first slice legible at
that level without claiming HA, CSI, ingress, or every cluster topology at
once.

## Objectives

- Define the first Port K3s slice around the shipped hosted control-plane
  vocabulary instead of inventing a second orchestration model.
- Sequence the smallest cluster outcome that proves workload orchestration
  without overcommitting to HA control planes, persistent volumes, or a full
  cluster manager.
- Reuse current host groups, deterministic placement, service visibility, and
  proof workflows wherever possible.
- Identify the minimum operator evidence that makes a K3s mission legible to
  humans and clearly bounded.

## Scope

- In scope: hosted-control-plane K3s bootstrap for one cluster per host group,
  fixed node roles, worker-node join flows, API or demo-workload reachability,
  kubeconfig handoff, and one canonical operator proof workflow.
- Out of scope: HA control planes, CSI or general persistent-volume support,
  multi-provider clusters, SSH-first cluster orchestration, ingress or
  load-balancer productization, and a generic Kubernetes distro abstraction.

## Success Criteria

- [ ] A first Port K3s slice is defined narrowly enough to plan as an epic and
  first voyage.
- [ ] The research identifies which current Port primitives can be reused for
  cluster bootstrap, placement, node lifecycle, and operator proof.
- [ ] The operator proof for the first slice is human-readable through a
  repo-local cluster bring-up, workload deploy, and reachability artifact.
- [ ] The remaining gap between the first hosted K3s lane and HA, persistent
  storage, SSH parity, or broader Kubernetes platform work stays explicit.

## Research Questions

- Should the first slice use one control-plane node plus workers, or another
  fixed hosted topology?
- How should Port bootstrap and join nodes: hosted guest exec, hosted service
  orchestration, or a small helper wrapped by the canonical CLI?
- What proof artifact best shows success to a human: kubeconfig plus demo app,
  a recorded proof workflow, or both?

## Open Questions

- How should Port materialize kubeconfig or API reachability without inventing
  a second operator command family?
- Should the first slice assume stateless workloads only until hosted storage
  support exists?

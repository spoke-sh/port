# K3s And Kubernetes Workloads — Brief

## Hypothesis

Port can grow into a higher-level workload platform if it offers a first-class
k3s lane built on the current hosted fleet, scheduler, and guest-operation
foundations instead of treating Kubernetes as an unrelated external add-on.

## Problem Space

Port already ships hosted fleet registration, placement, and multi-node service
primitives, but it stops well short of a full cluster product. The user wants
first-class Kubernetes support with k3s, and pointed to Slicer's HA k3s
examples as a reference for the human-facing outcome.

## Context

The current board already made one important boundary explicit: previous hosted
placement work intentionally excluded "a full Slicer-class cluster manager in
one voyage." That means the repo has enough substrate and fleet groundwork to
support a k3s research package, but not yet a cluster operator story.

Slicer's public examples show that HA k3s, multi-machine k3s, and autoscaling
k3s are the kind of outcomes humans immediately recognize as a platform
capability rather than a low-level hypervisor feature.

## Objectives

- Define what "first-class k3s support" means for Port's product shape.
- Sequence the smallest k3s slice that proves multi-node orchestration without
  overcommitting to a full Kubernetes product.
- Reuse current hosted placement, node ownership, and guest-operation contracts
  wherever possible.
- Identify the minimum operator evidence that makes a k3s mission legible to
  humans.

## Scope

- In scope: k3s cluster bootstrap, HA control-plane topology, worker-node join
  flows, node grouping, API exposure, and operator proof workflows.
- Out of scope: a general Kubernetes distro abstraction, multi-tenant cluster
  service, or every storage and CNI combination.

## Success Criteria

- [ ] A first Port k3s slice is defined narrowly enough to plan as an epic.
- [ ] The research identifies which current Port primitives can be reused for
  cluster bootstrap and node lifecycle.
- [ ] The operator proof for the first slice is human-readable, for example HA
  cluster bring-up, workload deploy, and service reachability.
- [ ] The remaining gap between "first k3s lane" and "full cluster platform"
  stays explicit.

## Research Questions

- Should the first k3s slice target one host, multiple hosts, or both?
- How should Port bootstrap and join nodes: guest exec, cloud-init style
  userdata, SSH orchestration, or a small helper?
- What proof artifact best shows success to a human: kubeconfig, demo app,
  video, or all three?

## Open Questions

- Should the first cluster lane ride on services and host groups, or treat
  machine groups as the primary abstraction?
- How much load-balancer or ingress setup belongs in the first Port k3s story?

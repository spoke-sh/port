# GPU Execution Support — Brief

## Hypothesis

Port can support higher-value workloads if accelerators become an explicit host
capability and machine-request contract, likely starting with a narrow
passthrough-oriented lane, instead of remaining absent from the product model.

## Problem Space

The current repo has no first-class GPU or accelerator surface. The user wants
GPU support, and external VM-platform examples suggest that accelerator work is
closely tied to host capability modeling, substrate choice, and placement.

## Context

Port already has:

- multiple substrate lanes,
- hosted node inventory and placement,
- and the first human-interest use cases that would benefit from accelerators,
  such as AI or cluster workloads.

What it does not yet have is any explicit model for GPU capability, passthrough
selection, scheduling, or operator proof.

## Objectives

- Define the smallest credible first GPU contract for Port.
- Decide which substrate and host capability boundary should carry the first
  GPU lane.
- Keep accelerator modeling consistent with hosted placement and future k3s
  work.
- Publish the explicit hardware and platform limits instead of implying generic
  "GPU support."

## Scope

- In scope: one explicit accelerator lane, host capability modeling, machine or
  service requests for GPUs, placement implications, and operator proof
  expectations.
- Out of scope: every GPU vendor, virtual GPU sharing, live migration, or a
  complete ML platform.

## Success Criteria

- [ ] The first GPU lane is bounded tightly enough to plan without pretending
  Port supports all accelerator scenarios.
- [ ] Host capability, placement, and substrate implications are explicit.
- [ ] The first user-visible proof for GPU support is concrete, for example one
  accelerated workload or one GPU-backed k3s demo.
- [ ] The follow-on relationship between GPU work and hybrid or k3s work is
  explicit.

## Research Questions

- Which substrate should own the first GPU lane?
- Should the first GPU contract be machine-focused, service-focused, or
  cluster-focused?
- How should Port model a request such as "one GPU" without leaking vendor
  detail everywhere?

## Open Questions

- Is the first worthwhile GPU lane local, hosted, or both?
- How much of the first accelerator story depends on storage and SSH or hybrid
  remote work landing first?

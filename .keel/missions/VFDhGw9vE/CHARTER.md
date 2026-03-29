# Ship Simple Port Cluster Bootstrap Surface - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver epic VFDhlRjOf so Port exposes one canonical cluster operator surface that can create, start, inspect, and return kubeconfig for a healthy single-node local K3s cluster without requiring operators to orchestrate raw `machine`, `guest exec`, join-token, or API-forward steps manually. | board: VFDhlRjOf |
| MG-02 | Move K3s bootstrap under Port ownership by making guest networking, offline artifact staging, and kubeconfig retrieval explicit Port responsibilities instead of infra-side glue or guest-side `curl get.k3s.io` scripts. | manual: inspect the Port CLI/docs and confirm the blessed workflow does not depend on ad hoc in-guest install commands from another repo |
| MG-03 | Keep the operator contract as simple as or simpler than Slicer's K3s workflow, with single-node local as the first clean lane and multi-node or AWS expansion modeled as explicit follow-on work. | manual: review `port --help`, cluster-facing help, and operator docs against the documented first-path experience |

## Constraints

- The first shipped contract must be cluster-oriented, not a longer wrapper
  around `port guest exec`.
- Single-node local is the first success criterion. Do not require a second VM
  or inter-node networking before Port can produce one healthy K3s cluster.
- Avoid guest-side network fetches during bootstrap. Port should own artifact
  staging and installation inputs.
- Keep multi-node, AWS, and richer cluster topology explicit follow-on scope
  unless they are required to make the first operator contract coherent.
- Preserve the canonical Port vocabulary and keep the resulting UX simpler than
  the current hosted-K3s proof path documented today.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when Port exposes one healthy local cluster contract with a usable
  kubeconfig and only manual operator verification remains
- YIELD to human when progress depends on product decisions about the final
  cluster command vocabulary, provider naming, or which multi-node and AWS
  behaviors belong in the first public contract

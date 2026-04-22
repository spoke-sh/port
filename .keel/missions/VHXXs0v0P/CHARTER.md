# Harden Hosted Control-Plane Placement And Inventory Fidelity - Charter

Archetype: Strategic

## Problem

During hosted prod recovery, Port could show machines as `malformed`, stall
`cluster status`, or block kubeconfig handoff even while node-agent truth and
the K3s API were healthy. The control plane still treats stored placement as
authoritative in synchronous read paths, repairs placement by writing on read,
and couples cluster readiness to kubeconfig guest-exec. That makes rollout and
auto-recovery trust the wrong layer.

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver epic `VHXXs1f1f` so hosted machine/service status, cluster status, and kubeconfig handoff stay truthful and non-blocking under placement drift, using live node-agent truth plus explicit degraded readiness instead of malformed read-path stalls. | board: VHXXs1f1f |
| MG-02 | Make placement and inventory stalls observable enough that operators and auto-recovery consumers can distinguish cache drift from real guest/runtime loss quickly. | manual: run hosted control-plane status/list probes plus targeted tests that prove repaired placement, timeout isolation, and degraded readiness detail |

## Constraints

- Keep one canonical hosted control-plane and CLI surface; do not introduce a
  second debug-only API or push recovery logic downstream into `infra`.
- Treat stored placement as a cache and live node-agent/runtime truth as the
  recovery authority; do not add new synchronous write-on-read repair paths.
- Keep scope bounded to hosted placement, readiness, and observability; do not
  absorb scheduler, storage, or Flux reconciliation work into this mission.

## Halting Rules

- DO NOT halt while hosted machine or service status can still return
  `malformed` or wedge solely because stored placement drifted.
- HALT when epic `VHXXs1f1f` is complete and only human verification of the
  repaired hosted rollout/runtime proof remains.
- YIELD to human if the fix requires changing the external hosted auth model or
  introducing a second operator surface.

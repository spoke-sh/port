# Hosted Control Plane And Operator Surface — Brief

## Hypothesis

Port's next highest-leverage slice is no longer another substrate-only contract.
The missing step toward the user's SlicerVM-parity goal is a hosted
control-plane foundation that makes machine inventory, status/stop, auth, API,
node or host-group placement, and guest-operation brokerage real product
surfaces instead of design-only docs.

## Problem Space

The current board is empty after completing the substrate, PVM, and AVF
contracts, but the user objective is still incomplete. Port still lacks the
hosted/node-agent/control-plane capabilities that make Slicer productized:

- remote inventory, status, and stop
- an authenticated API surface
- node and host-group vocabulary
- monitoring and machine visibility
- secrets and higher-level service or sandbox flows
- productized hosted CLI and docs rather than local-only runtime ownership

## Context

Port had already documented several substrate and hosted concepts, but the next
meaningful gap was no longer another capability matrix. The board needed a
research package that collapsed the broad Slicer-parity request into the
smallest coherent hosted-control program that could become executable work.

## Objectives

- Identify the first hosted-control epic that materially changes Port from a
  local runtime plus docs into a real remote product surface.
- Sequence auth, inventory, node ownership, guest brokerage, and the next layer
  of operator features into implementation-ready voyages.
- Explicitly defer downstream features that should not land before the hosted
  foundation is stable.

## Scope

- In scope: authenticated API identity, node or host-group vocabulary, hosted
  `machine list|status|stop`, guest-connect brokerage, and the sequencing of
  monitoring, secrets, services, and sandboxes on top of that base.
- Out of scope: fully implementing all downstream operator features in the
  research slice or revisiting substrate-level feasibility that earlier
  bearings already covered.

## Success Criteria

How will we know if this research was valuable?

- [x] Identify the smallest coherent hosted-control epic that should come next
      instead of treating every missing Slicer feature as one story.
- [x] Produce a recommendation that orders API/auth/inventory, guest
      brokerage, monitoring, secrets, and service or sandbox work into
      implementation-ready voyages.

## Research Questions

- What is the first executable hosted-control slice that materially changes Port
  from design-only hosted docs into a real product surface?
- Which missing operator features depend on the same API/auth/node-agent
  foundation and should therefore be sequenced together?
- What should be deferred until after the first hosted-control foundation is
  landed?

## Open Questions

- What persistence model should own the first hosted inventory and lifecycle
  state once the API exists?
- How much monitoring or observability should land in the same first hosted
  foundation versus the immediately following voyage?
- Which authentication surface is smallest while still being compatible with a
  later SDK and multi-node control plane?

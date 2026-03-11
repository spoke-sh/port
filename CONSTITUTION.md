# Constitution

This document defines the non-negotiable rules for `port`. When there is a
tradeoff between convenience and coherence, these rules win unless a higher
priority board artifact overrides them explicitly.

## 1. One Canonical Operator Surface

`port` is the product surface. User-facing docs and examples should use
`port`, not repo-local cargo wrappers.

- `artifacts`, `machine`, `guest`, and `service` are the canonical families.
- Hosted workflows must reuse the same verbs instead of inventing a hosted-only
  command tree.

## 2. One Machine Model Across Lanes

Local and hosted execution share the same machine identity and lifecycle model.

- The runtime owner may change between local runtime roots and hosted
  control-plane plus node-agent routing.
- The operator vocabulary must not change just because the owner changes.

## 3. Explicit Substrate And Protection Lanes

Port models substrate and protection mode directly.

- Firecracker, Cloud Hypervisor, and AVF are explicit substrate lanes.
- `standard` and `pvm` are explicit protection contracts.
- Unsupported combinations fail fast; they do not silently fall back.

## 4. Fail Fast Beats Hidden Compatibility

Port uses hard cutover by default.

- Replace old contracts in the same slice when a new canonical contract lands.
- Do not add compatibility aliases, soft deprecations, or hidden fallback
  logic unless a story explicitly requires them.

## 5. Runtime Ownership Must Stay Inspectable

Operators should be able to tell who owns runtime state.

- `port doctor`, `machine status`, and `service status` should expose explicit
  routing and runtime detail.
- Secrets, placements, and health state must remain attributable to a single
  runtime owner.

## 6. Configuration And Docs Are Product Contracts

The checked-in sample config and root docs are part of the product surface.

- `examples/port.toml` is the canonical repo example.
- Root contracts such as `CONFIGURATION.md` and `ARCHITECTURE.md` should stay
  current with shipped behavior.
- Top-level help surfaces stay short; detail lives in canonical docs.

## 7. Evidence-Backed Delivery

Implementation is not done at a clean compile.

- Every functional slice needs verification through tests, command proofs, or
  explicit manual evidence.
- Board state, proofs, and docs should move together.

## 8. Local And Hosted Paths Are First-Class

Hosted features are not second-tier documentation promises.

- If Port claims a hosted workflow, it should have real control-plane and
  node-agent ownership plus proof.
- Local workflows remain first-class even as hosted capability expands.

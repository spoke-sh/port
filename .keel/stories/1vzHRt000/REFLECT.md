---
created_at: 2026-03-08T10:07:21
---

# Reflection - Model Pvm Node Capability Contract

## Knowledge

- [1w04a0000](../../knowledge/1w04a0000.md) Prefer Per-Ac Verify Annotations Over Shared Proof Blocks

## Observations

The cleanest implementation path was to keep the rich local
`FirecrackerPvmLaneContract` intact, introduce one smaller hosted-node
capability contract plus a shared capability-state enum, and then reuse that
state vocabulary in both places.

The proof-first loop worked as intended here. The failing tests forced the
exact surface area: model state, sample config, hosted inventory serialization,
and nothing broader. The only friction was `keel story record` metadata drift
when a shared verify block was present, which is why the per-AC annotation form
is now the preferred pattern.

---
created_at: 2026-04-12T17:44:58
---

# Reflection - Record Hosted Worker Stability Soak Proof

## Knowledge

- [VGct92Y9v](../../knowledge/VGct92Y9v.md) Hosted Proof Harnesses Must Seed Registrations Before Control-Plane Start

## Observations

- The repository already had a strong HA/failover proof script; the work was mostly hardening it so it matched current hosted-control-plane state handling and the post-managed-service cluster access contract.
- The main surprises were operational: the proof needed a unique temporary control-plane name to avoid `.port/hosted/demo` collisions, and manual registered-node state had to exist before `control-plane serve` so route resolution would treat the nodes as live.
- Once the harness matched those constraints, the artifact was stable and produced a concise before/after proof that the hosted worker stayed visible through a simulated control-plane guest replacement without depending on private local state.

---
created_at: 2026-04-12T17:35:58
---

# Reflection - Enforce Managed Service Ownership For Hosted K3s

## Knowledge

- [VGct0mwbD](../../knowledge/VGct0mwbD.md) Hosted Proof Harnesses Need Isolated Control-Plane State

## Observations

- Replacing the legacy detached bootstrap in the proof script was straightforward once the runtime/CLI contract was traced from `bootstrap_hosted_k3s_cluster` and hosted cluster status printing.
- The main friction was harness realism rather than product logic: the proof had to honor the repo's external `CARGO_TARGET_DIR`, avoid collisions with existing hosted control-plane state, and disable guest NIC setup because this mocked proof only needs managed-service and vsock surfaces.
- Running the renderer itself was the right verification gate because it exposed protocol-order mismatches immediately and confirmed the canonical managed-service workflow end to end once the harness matched runtime expectations.

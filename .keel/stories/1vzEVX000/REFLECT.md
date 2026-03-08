---
created_at: 2026-03-08T09:13:28
---

# Reflection - Publish Hosted Demo Workflow And Evidence

## Knowledge

- [1w03mg000](../../knowledge/1w03mg000.md) Prefer Reusable Demo Scripts Over Test-Only Proof For Operator Workflows

## Observations

The story was smaller than the transport cutover, but it still benefitted from the same verification discipline. The new reusable `scripts/hosted-demo.sh` proof made the documentation tighter because every step had to be runnable outside the test harness.

What went well was using the same deliberate split between server config and client config from the previous story. That let the demo evidence prove the live HTTP path again instead of quietly regressing to runtime-root coupling.

The main difficulty was keeping the operator docs honest about the limits that still remain. The demo is real and reproducible, but it is still explicitly a single-node repository-local lane, with hosted `copy` depending on node-visible host paths and hosted `forward` still relying on the repo-local listener lifecycle.

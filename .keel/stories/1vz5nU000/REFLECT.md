---
created_at: 2026-03-07T23:10:00
---

# Reflection - Implement Hosted Control Plane Runtime Path

## Knowledge

## Observations

- The first runnable hosted slice did not need a networked control plane to be
  real. It needed a concrete node-agent-owned runtime root that the existing
  machine surfaces could route through.
- Making `machine list` config-aware was the key behavioral shift. Hosted
  machines can now appear alongside local runtime directories without inventing
  a second inventory command family.
- Surfacing unresolved hosted ownership as `malformed` is better than dropping
  those machines from the list. It keeps control-plane and node-inventory
  mismatches visible to the operator.

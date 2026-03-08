---
created_at: 2026-03-07T20:35:00
---

# Reflection - Define Hosted Node Inventory Model

## Knowledge

## Observations

- Port needed a distinct hosted node inventory layer above `hosts` because
  `hosts` describe raw execution environments while hosted placement needs
  capability and membership semantics that later scheduling work can reuse.
- Keeping host-group placement to explicit membership is the right first step.
  It makes the grouping contract real without pretending Port already has a
  scheduler.
- Deriving node and host-group contracts from existing host and control-plane
  data keeps one ownership vocabulary across auth, inventory, lifecycle, and
  later guest-bridge work.

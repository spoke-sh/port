---
created_at: 2026-03-07T21:55:00
---

# Reflection - Define Hosted Guest Bridge Attach Contract

## Knowledge

## Observations

- Hosted guest transport needed a first-class contract above the existing
  machine control tokens so future runtime work can preserve one canonical
  `guest` surface instead of inventing separate hosted verbs.
- The important implementation boundary is the attach path: control plane
  authorization, node-agent transport ownership, then unchanged guest protocol
  frames into the in-guest agent.
- Operator docs need to separate the attach contract from runtime availability.
  Saying that hosted guest attach is modeled today but not runnable yet prevents
  the docs from overclaiming the current hosted lane.

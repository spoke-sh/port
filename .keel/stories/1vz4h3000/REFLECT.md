---
created_at: 2026-03-07T21:35:00
---

# Reflection - Define Hosted Machine Lifecycle Surface

## Knowledge

## Observations

- Hosted lifecycle needed its own explicit contract layer above node inventory
  so future runtime work can preserve the canonical `machine` verbs instead of
  splitting into separate local and hosted command families.
- The important distinction for operators is not just the route token. It is
  whether the lifecycle surface is modeled versus runnable today, so the help
  text and hosted docs need to say that directly.
- Deriving hosted summary, status, and stop from the shared machine control
  contract keeps one ownership and routing vocabulary across local runtime,
  hosted planning, and later node-agent execution work.

---
created_at: 2026-03-07T18:02:00
---

# Reflection - Publish Hosted Node Agent Contract

## Knowledge

- [1vz3N8000](../../knowledge/1vz3N8000.md) Hosted Port Should Broker The Guest Protocol, Not Replace It

## Observations

- The lifecycle story made the hosted boundary clearer. Once `list/status/stop`
  were explicitly runtime-root based, it became obvious that a future node
  agent should own that same host-local state instead of the short-lived CLI.
- The hosted contract needed to stay concrete. Naming the exact owners for
  lifecycle, transport brokering, and inventory turned the story from general
  architecture prose into an implementation-facing contract.
- Surfacing the contract in `README`, operator docs, and CLI help matters even
  before a daemon exists. Hosted Port should be discoverable as a real planned
  lane, not hidden tribal knowledge.

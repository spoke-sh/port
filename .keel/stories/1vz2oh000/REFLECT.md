---
created_at: 2026-03-07T18:02:00
---

# Reflection - Publish Hosted Node Agent Contract

## Knowledge

### 1vz3N8000: Hosted Port Should Broker The Guest Protocol, Not Replace It
| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When Port extends from local runtime ownership into a hosted node-agent plus control-plane architecture |
| **Insight** | Port already has a usable guest protocol and CLI vocabulary. The hosted layer should add routing, ownership, and auth around that protocol instead of inventing a second hosted-only guest API. |
| **Suggested Action** | Keep hosted design and implementation work centered on tunneling the existing guest protocol through node agents and control-plane sessions, with the CLI remaining the same surface in local and hosted modes. |
| **Applies To** | docs/hosted.md, crates/port-runtime/**, crates/port-cli/**, future hosted-control crates |
| **Observed At** | 2026-03-08T02:02:00Z |
| **Score** | 0.89 |
| **Confidence** | 0.94 |
| **Applied** | yes |

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

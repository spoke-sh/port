---
created_at: 2026-03-08T07:06:15
---

# Reflection - Implement Control Plane Serve Path

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzElr000: Title
| Field | Value |
|-------|-------|
| **Category** | code/testing/process/architecture |
| **Context** | describe when this applies |
| **Insight** | the fundamental discovery |
| **Suggested Action** | what to do next time |
| **Applies To** | file patterns or components |
| **Linked Knowledge IDs** | optional canonical IDs this insight builds on |
| **Observed At** | RFC3339 timestamp (e.g. 2026-02-22T12:00:00Z) |
| **Score** | 0.0-1.0 (impact significance) |
| **Confidence** | 0.0-1.0 (insight quality) |
| **Applied** | |
-->

## Observations

- A live hosted control-plane slice can land before the real node-agent server
  if the control plane is written as a thin authenticated proxy to explicit
  node bindings. That keeps the layering honest and lets the next story focus
  on the node side without reworking the public route surface.
- The `:stop` suffix in Port's canonical HTTP contract does not fit axum's
  normal route parameter grammar. The clean workaround is to route `POST
  /v1/machines/{machine}` and strip the `:stop` suffix inside the handler
  rather than weakening the public contract.
- Binding node-agent endpoints explicitly on the control-plane CLI is a good
  demo-lane compromise. It makes the hosted transport real now without forcing
  premature durable registration or scheduler work into the same story.

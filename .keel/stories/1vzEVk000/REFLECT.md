---
created_at: 2026-03-08T07:12:27
---

# Reflection - Implement Node Agent Serve Path

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzErr000: Title
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

- Reusing the existing runtime by localizing the machine's host connection in a
  cloned config is the right seam for the node agent. It keeps the node-agent
  server thin and avoids creating a second execution stack for status, stop,
  or guest operations.
- The internal node-agent routes can stay close to the public control-plane
  contract while still using node-specific auth and route context. That made
  the control-plane proxy and the real node-agent server line up cleanly.
- Hosted guest forward can work in the first node-agent slice by opening the
  listener on the node host and returning the bound listen address, without
  forcing the control plane to become a byte-stream proxy yet.

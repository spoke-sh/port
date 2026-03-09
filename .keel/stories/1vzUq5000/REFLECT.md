---
created_at: 2026-03-09T01:33:13
---

# Reflection - Define Durable Hosted Registry Contract

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzW37000: Title
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

- The contract work was already live in the model and hosted-protocol layers, but the board still pointed at nonexistent generic test names. Closing the story required rebasing evidence onto the concrete tests that actually prove the contract today.
- The cleanest proof split was model plus hosted-protocol tests for registration shape and serialization, then runtime merge tests for imported inventory and error detail. That matches where the responsibilities actually landed after the voyage evolved.
- When board work gets overtaken by later implementation slices, the expensive part is usually evidence hygiene rather than code. It is still worth fixing immediately because stale backlog items make later voyage-closure criteria look blocked even when the product behavior is already there.

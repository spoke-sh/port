---
created_at: 2026-03-08T21:20:42
---

# Reflection - Define Managed Service Execution Contract

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzS6k000: Title
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

- The existing `service` CLI integration test already created hosted runtime
  fixtures, so extending it with runtime-state assertions was a cleaner proof
  surface than inventing a new demo path for a contract-only slice.
- The smallest safe contract cut was shared protocol plus runtime/route
  identity, not execution. Adding the service runtime record path and service
  route context now keeps the later supervisor story additive instead of
  forcing a second status shape.
- `keel story record` still rewrites proof links incorrectly on multi-AC
  stories, so the story README needs an immediate coherence check after
  recording evidence.

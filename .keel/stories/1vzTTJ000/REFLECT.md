---
created_at: 2026-03-08T23:43:08
---

# Reflection - Route Hosted Machine Launch Through Registered Nodes

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzUKa000: Title
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

Hosted machine launch already had the right control-plane routing seam, so the
incremental change was to intercept successful launch responses and persist
selected-node placement without widening the CLI surface. The main difficulty
was test isolation: both runtime and CLI suites share `.port/hosted/<control-plane>`
state, so new placement artifacts exposed cross-test contamination that had not
been visible before. Using unique control-plane names for the new runtime proofs
kept the full workspace suite stable while preserving deterministic placement
coverage.

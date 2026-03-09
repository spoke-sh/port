---
created_at: 2026-03-09T03:04:50
---

# Reflection - Define Hosted Standard Placement Contract

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzXTm000: Title
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

Adding provider and host identity to the hosted summary contract made the later
runtime-routing story cleaner because placement decisions no longer need to
re-derive that context from machine and node state.

The main trap was negative-path fixture setup. Removing a node from the sample
config also breaks any host-group membership that references it, so tests that
intend to exercise missing-host inventory need to clean both inventory and
group membership or they will fail earlier in hosted inventory construction.

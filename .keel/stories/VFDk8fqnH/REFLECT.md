---
created_at: 2026-03-28T21:03:22
---

# Reflection - Add Cluster CLI And Config Contract

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### VFE3VxwqZ: Title
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

The implementation stayed cleanest when the new operator-facing cluster
contract was introduced beside the existing hosted `k3s_clusters` substrate
instead of trying to fold both concerns together in one slice. That kept the
first story focused on config, CLI, and fail-fast boundaries.

The main difficulty was regression hygiene in tests that derive narrower hosted
or AVF fixtures from `PortConfig::sample()`. Once the sample gained a local
cluster, those fixtures had to clear `clusters` explicitly or they became
invalid in ways unrelated to the behavior under test.

---
created_at: 2026-03-11T11:37:50
---

# Reflection - Implement Secret Backend And Materialization

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### VDaMCuO7M: Title
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

- Replacing the secret value with runtime-owned metadata plus a separate
  runtime-file backend let the storage contract hard-cut over without changing
  the `port service secret` or `port service apply` command surface.
- The status projection needed an explicit `secret_sources` structure because
  the existing `secret_bindings` field only showed logical bindings, not
  backend/materialization provenance for repeated `service status` reads.
- `keel story record` still misrendered this story's acceptance metadata, so
  the README acceptance markers and proof links had to be repaired manually
  even though the proof logs were recorded correctly.

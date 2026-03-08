---
created_at: 2026-03-08T15:35:52
---

# Reflection - Define Streamed Guest Session Contract

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzMj2000: Title
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

- The stream contract lands cleanly when the protocol, hosted route model, and
  SDK builder all move together in the same slice. Trying to stage only one
  layer first leaves the next story to guess lifecycle or path semantics that
  should already be encoded in types.
- The workspace hygiene gate caught a real test-harness regression that narrow
  package tests missed. The hosted server helper changed from a best-effort
  fire-and-forget spawn into an explicit readiness/error contract, and the full
  workspace run was what exposed the unfinished call sites.
- For long-lived hosted transport tests, the helper should report bind failures
  over a channel instead of panicking in a detached thread. That keeps
  readiness failures attributable to the caller and avoids hidden port-race
  flakes in later stories that build on the same hosted test harness.

---
created_at: 2026-03-09T02:41:02
---

# Reflection - Publish Hosted Artifact Mobility Workflow

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzX6k000: Title
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

- The build-first hosted CLI proof exposed a real runtime bug that the smaller
  synthetic artifact test missed: Axum's default request-body limit blocked
  hosted artifact uploads for real kernel-sized payloads until the push route
  explicitly disabled the default body limit.
- Publishing the hosted workflow required keeping three operator surfaces in
  lockstep: README for discovery, `docs/artifacts.md` for the exact control
  plane ownership and auth contract, and CLI `--help` for immediate
  learnability.

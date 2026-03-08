---
created_at: 2026-03-08T06:56:10
---

# Reflection - Define Hosted HTTP Control Contracts

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzEc6000: Title
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

- A dedicated `port-hosted-protocol` crate is the right seam for live hosted
  transport work because it keeps HTTP routes and auth/header rules out of
  `port-model` while remaining lighter-weight than `port-runtime`.
- Repointing `port-sdk` at the shared contract immediately exposed route and
  header drift risk, which is exactly why the shared crate needed to exist
  before control-plane and node-agent servers land.
- `keel story record` still misassigns proof pointers on multi-AC stories, so
  the story README needs a quick manual check before submission.

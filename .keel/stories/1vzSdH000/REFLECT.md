---
created_at: 2026-03-08T22:41:53
---

# Reflection - Publish Multi-Node Hosted Service Workflow

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzTNJ000: Title
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

- The workflow and limit proof was easiest to stabilize by testing for durable command fragments rather than exact full help lines. `clap` wraps long `after_help` output, so verification should grep the meaningful fragments instead of one huge line.
- The upgraded `keel verify` path expects absolute script paths in the story README. Relative `bash .keel/...` entries worked manually from the repo root but failed under `keel verify run` with exit code `127`.
- `keel story record` still rewrites the first acceptance proof link incorrectly, so README proof comments and reflection files still need a manual inspection before submission.

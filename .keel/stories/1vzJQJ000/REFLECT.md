---
created_at: 2026-03-08T14:18:01
---

# Reflection - Publish Prepared Pvm Operator Workflow

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzLVh000: Title
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

The user-facing gap here was stale documentation, not missing code. The safest
way to fix it was to turn the docs into executable promises first: one proof
script checked the published help and docs for the live prepared-node wording,
and a second proof script ran the actual hosted PVM launch plus the preserved
standard-lane launch through raw `port` commands.

The prepared-node CLI proof needs a temporary config overlay because the sample
config intentionally keeps `cloud-aws` on the standard lane. Keeping that
overlay logic in the story-local proof script avoided changing the canonical
sample defaults just to make the documentation slice pass.

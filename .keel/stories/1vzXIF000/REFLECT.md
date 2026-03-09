---
created_at: 2026-03-09T03:24:06
---

# Reflection - Route Standard Cloud Launch Through Hosted Runtime

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzXmQ000: Title
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

- The hosted runtime path was already capable of standard-lane placement once the early provider-guidance rejection was removed; the missing work was proving that the same route contract used for PVM also holds for `cloud-generic`, `cloud-aws`, and `cloud-gcp`.
- The main regression risk was operator visibility, not launch itself. `machine status` and `machine stop` render hosted route context inside the `detail` field, so the story needed explicit assertions that provider and selected-node detail survive launch and subsequent lifecycle commands.
- CLI proof coverage caught the important product-surface distinction between structured runtime state and rendered operator output. Keeping both runtime and CLI round-trip tests in the story evidence prevented a false pass from runtime-only coverage.

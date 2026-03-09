---
created_at: 2026-03-09T00:11:01
---

# Reflection - Publish Registered Hosted Machine Workflow

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzUlZ000: Title
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

- The real hosted runtime already supported node self-registration; the drift
  was in operator-facing surfaces that still taught `--node-binding` as the
  default workflow.
- `clap` help wrapping makes long-string assertions brittle. Stable substring
  checks plus a real CLI proof script were more reliable than matching full
  prose lines.
- The repository-local hosted demo script is now a useful regression proof for
  registration, `machine list`, and explicit hosted limits, not just guest
  transport.

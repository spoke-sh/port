---
created_at: 2026-03-08T22:06:47
---

# Reflection - Define Host Group And Scheduler Contracts

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzSpL000: Title
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

- The repo already had host groups as an explicit membership concept, so the
  real gap was carrying scheduler policy through the same shared model and
  runtime surfaces rather than inventing a new placement abstraction.
- Keeping host-group context on rejected placements mattered immediately:
  without it, the hosted route context lost scheduler detail exactly when
  operators most need it. The contract should preserve group metadata even when
  every candidate node is rejected.
- The upgraded `keel` parser is strict about Markdown tables and inline `|`
  tokens inside requirement text. Planning artifacts need phrasing like
  "`service` and `sandbox`" or separate verbs instead of pipe-delimited command
  shorthand.

---
created_at: 2026-03-08T06:38:01
---

# Reflection - Add Hosted Secrets Services And Sandboxes

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzEKX000: Title
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

- The coherent first cut was one canonical `service` family, not separate
  hosted-only service and sandbox command trees. Modeling sandboxes as
  `--kind sandbox` keeps the future SDK and API surface aligned with the same
  service verbs.
- The existing runtime and guest foundations were sufficient for a first
  secrets/services slice, but not for real hosted execution. Persisting desired
  state, guest command, secret bindings, and routing context under the resolved
  runtime owner made the current boundary explicit instead of pretending a
  node-agent execution engine already exists.
- Secret handling needed an equally explicit limitation callout. The current
  bootstrap store is a runtime-owned JSON file, which is acceptable only if the
  docs and help text say plainly that hardened secret backends remain follow-on
  work.

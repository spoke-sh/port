---
created_at: 2026-03-07T21:11:27
---

# Reflection - Implement Hosted Guest Operations Runtime Path

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vz6QJ000: Title
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

- The hosted guest slice did not need a new guest protocol or a hosted-only
  CLI. It needed the hosted driver to resolve the owning node runtime root and
  reuse the existing local guest transport beneath the same `guest` verbs.
- Routing hosted guest operations through configured node `runtime_root`
  directories keeps the first hosted runtime slice inspectable and testable
  without pretending a full networked control plane already exists.
- The important operator boundary is not just that hosted guest commands now
  work. The docs and help text also need to say what still remains explicitly:
  detached forwarding, Unix-socket forwarding, monitoring, services, sandboxes,
  and SDK work are follow-on slices, not hidden partial behavior.

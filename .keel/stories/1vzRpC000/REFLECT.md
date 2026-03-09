---
created_at: 2026-03-08T21:29:06
---

# Reflection - Implement Guest-Agent Managed Process Supervisor

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzSEs000: Title
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

- The guest-agent slice could stay self-contained by using the new managed
  service protocol plus runtime-record files under the guest root, which kept
  the later hosted node-agent story additive instead of forcing shared mutable
  state into `port-runtime` early.
- Redaction is easiest to prove at the log-writer boundary. Capturing managed
  stdout/stderr through pipes and replacing injected secret values before they
  hit disk avoids leaking them into status or runtime-record evidence.
- The existing mixed transport tests already covered `exec|copy|pty|logs|forward`,
  so the new story only needed one focused lifecycle test to prove the managed
  path without broadening the blast radius.

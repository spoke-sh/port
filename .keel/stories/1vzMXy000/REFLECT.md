---
created_at: 2026-03-08T16:17:37
---

# Reflection - Implement Hosted Streamed Copy Transport

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzNNR000: Title
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

- Reusing the existing guest copy protocol over hosted HTTP avoided inventing a
  second hosted-only file-transfer contract. The control plane now proxies raw
  copy-stream bytes while the node agent reconstructs a hosted response from
  the same guest copy semantics already used locally.
- The main friction was verification bookkeeping rather than runtime logic:
  `keel story record` keyed both acceptance criteria to the same `SRS-03`
  marker and rewrote the first proof link to `ac-2.log`. The README pointer had
  to be corrected manually after recording proof.
- The hosted copy path is protocol-correct but still buffered at the HTTP relay
  boundaries because the current Axum/reqwest helpers collect request and
  response bodies. That is acceptable for copy, but hosted forward still needs
  a true streaming relay rather than the buffered copy pattern.

---
created_at: 2026-03-08T06:26:30
---

# Reflection - Add Hosted Monitoring And Top

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzE9O000: Title
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

- Hosted monitoring did not need a hosted-only command family. Extending the
  canonical `machine` surface with `monitor` and `top` kept the operator model
  coherent while reusing the same local-versus-hosted driver boundary as
  `status` and `stop`.
- The useful runtime ownership signal was already present in the hosted runtime
  slice. The extra work here was exposing detached forward manifests and live
  process inspection as first-class monitoring data instead of inventing a new
  control-plane abstraction too early.
- The docs boundary matters as much as the code: `monitor` and `top` are now
  shipped runtime-inspection surfaces, but they are not yet a full metrics,
  secrets/services, or sandbox product. That distinction needs to stay explicit
  in help text and hosted docs.

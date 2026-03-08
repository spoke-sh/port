---
created_at: 2026-03-08T11:44:48
---

# Reflection - Gate Hosted Pvm Placement

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzJ7Q000: Title
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

- The right seam was the hosted machine summary contract, not the transport
  layer. Once placement eligibility and rejection reasons were modeled there,
  the control plane, SDK, and local hosted-launch guardrail could all share the
  same denial detail.
- Hosted `status` needed to degrade into a malformed machine record instead of a
  transport error. That keeps unplaceable hosted machines visible to operators
  and avoids teaching them that inventory gaps mean the machine disappeared.
- The most valuable proof was a CLI-level hosted status plus launch denial
  sequence. The crate tests proved the internals, but the CLI test locked the
  operator-facing wording and state transition down.

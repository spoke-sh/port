---
created_at: 2026-03-09T10:04:49
---

# Reflection - Bridge Cloud Hypervisor Guest Sessions

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vze2D000: Title
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

- Cloud Hypervisor guest enablement was blocked by two concrete omissions rather than a new API gap: the local driver refused guest attach outright, and the launch path was not carrying the guest init boot arguments or vsock socket state.
- Reusing the existing runtime socket first, then the shared live-vsock tunnel path, kept the CLI and hosted HTTP routes unchanged. The only hosted-specific adjustment needed in proof fixtures was to advertise Cloud Hypervisor in node substrate capabilities so placement stayed honest.
- The `keel story record` and `reflect` commands still leave story markdown partially scaffolded. Future slices should expect a manual cleanup pass before submit so the board reflects the actual proof commands and reflection text.

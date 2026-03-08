---
created_at: 2026-03-07T18:51:22
---

# Reflection - Define Hosted Machine Inventory Contract

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vz4Ek000: Title
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

- The hosted/lifecycle story only became implementation-ready once the local
  and hosted tokens lived in `port-model` as typed contract data rather than as
  prose in `docs/hosted.md`.
- Threading that contract through `MachineStatus` and `StopResult` gave the
  local CLI an operator-visible vocabulary that future hosted drivers can fill
  without renaming `machine list`, `status`, or `stop`.
- The existing runtime-root lifecycle code was stable enough that this slice
  could focus on contract publication and CLI discoverability instead of
  changing local execution behavior.

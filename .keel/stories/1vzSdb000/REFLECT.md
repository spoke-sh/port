---
created_at: 2026-03-08T22:35:25
---

# Reflection - Surface Placement State Through Canonical Service Commands

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzTH3000: Title
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

- Hosted service routing cannot rely on generic hosted machine resolution once a concrete node has already been selected. The placement-aware control plane work only became correct after the live service refresh and stop helpers wrote runtime records directly under the node-owned `runtime_root`.
- Stored service definitions are the right source of truth for hosted `service list|status|stop`. When a node binding disappears, surfacing the stored node and host-group detail keeps the canonical service commands operator-usable instead of collapsing into generic hosted routing failures.
- `keel story record` again rewrote the AC-01 proof link to `ac-2.log`, so story artifacts still need a manual README inspection before submission.

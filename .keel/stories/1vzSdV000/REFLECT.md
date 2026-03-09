---
created_at: 2026-03-08T22:23:37
---

# Reflection - Implement Hosted Service Placement Scheduler

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzT5d000: Title
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

The hosted service path needed two different fixes to become trustworthy. First,
the control plane had to stop treating hosted `service apply` like generic
machine routing and instead parse the request, require a target host group, and
pick a node in deterministic sorted order. Second, the node-side metadata write
could not rely on generic hosted machine resolution once multiple equal nodes
were available, so the service runtime context now honors the selected runtime
root and host group when recording placement.

The full workspace test pass also surfaced an AVF regression outside the active
story: AVF launch preflight still required `firecracker` on `PATH` for the
standard lane. Fixing that in the same slice was necessary to keep the hygiene
gate green and reinforced that scheduler work needs full-suite verification,
not only targeted tests, because unrelated substrate assumptions can break the
global product surface.

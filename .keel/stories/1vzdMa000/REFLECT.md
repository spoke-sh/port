---
created_at: 2026-03-09T10:15:10
---

# Reflection - Route Hosted Cloud Hypervisor Lifecycle

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzeCE000: Title
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

- The hosted Cloud Hypervisor lifecycle path was already substrate-generic once the earlier driver extraction and guest-session routing were in place. The missing work for this story was proof coverage that exercised hosted launch, status, stop, and placement rejection explicitly against a Cloud Hypervisor machine.
- The main surprise was in the rejection output. Correct failure messages still mention `firecracker` because the rejected node explains that it only advertises the Firecracker substrate. The right proof is therefore explicit Cloud Hypervisor placement context and rejected-node detail, not a blanket assertion that `firecracker` never appears in the message.

---
created_at: 2026-03-09T08:24:53
---

# Reflection - Add Pvm Artifact Mobility Workflow

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzcTV000: Title
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

- The implementation path was already correct for PVM artifact selection; the real gap was evidence and discoverability. The CLI artifact tests were only modeling `firecracker/standard`, which left the shipped PVM claims under-proven.
- Making the test harness preserve each variant's actual `protection_mode` exposed the intended contract clearly and let one round-trip test cover both kernel and guest-image mobility without any fallback logic.
- `keel story record` updated proof metadata but did not fully normalize the story README, so the acceptance checklist still needed a manual cleanup pass before submission.

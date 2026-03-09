---
created_at: 2026-03-09T09:34:55
---

# Reflection - Define Cloud Hypervisor Contract And Doctor Checks

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzdZH000: Title
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

- The smallest durable slice was not a driver stub; it was the contract boundary. Adding explicit Cloud Hypervisor artifact variants and a concrete sample machine exposed the missing `doctor` lane checks immediately.
- `port doctor` needed substrate-specific checks in addition to the generic machine contract. Reusing the AVF pattern kept the implementation narrow and made the new lane visible without pretending the driver already existed.
- The evidence loop also surfaced board hygiene drift: `keel story record` does not reconcile checkbox state or reflection content automatically, so the story markdown still needs a manual cleanup pass before submission.

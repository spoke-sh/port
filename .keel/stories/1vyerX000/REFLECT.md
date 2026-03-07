---
created_at: 2026-03-06T16:01:18
---

# Reflection - Publish Cloud Support Matrix

## Knowledge

- [1vyeP0000](../../knowledge/1vyeP0000.md) Anchor Platform Guidance On `port doctor`

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vyf6c000: Title
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

- The cloud lane only became understandable once the same support matrix appeared in three places at once: `port --help`, the README, and a dedicated cloud/operator doc.
- Keeping the docs anchored on the actual commands `port doctor` and `port machine launch` prevented the cloud text from drifting into promises about remote launch that the runtime still does not make.
- Recording the PVM drop decision in shipped docs, not only in the research bearing, closes the loop between current research and what a new operator will infer from the product surface.

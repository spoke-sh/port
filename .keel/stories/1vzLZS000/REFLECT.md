---
created_at: 2026-03-08T14:41:06
---

# Reflection - Define Avf Machine Contract And Doctor Checks

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzLs2000: Title
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

- The model contract and doctor contract needed to move together. Adding AVF
  validation tests first exposed that hosted-control-plane AVF hosts were still
  accepted even though the lane is explicitly local-only today.
- `port doctor` was still framed around Firecracker/Linux, so the shipped AVF
  slice needed both dedicated AVF checks and operator-note updates to avoid
  implying that macOS always means "use Linux instead."
- Board hygiene caught a real planning problem in the AVF voyage. The driver
  story mixed an early runtime requirement with a late rollout-preservation
  requirement, which created a dependency cycle across the remaining stories.
  Reassigning that preservation proof to the workflow-doc story restored the
  intended execution order: contract and doctor, then driver, then guest
  transport, then published workflow.

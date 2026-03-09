---
created_at: 2026-03-09T10:26:11
---

# Reflection - Publish Cloud Hypervisor Operator Workflow

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzeMt000: Title
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

- The shipped Cloud Hypervisor lane was broader than the published surface:
  lifecycle and guest-session proofs already existed locally and through the
  hosted control path, but the top-level README, cloud docs, operators guide,
  and CLI help still described the lane as planned.
- The checked-in sample config needs to stay on the standard Firecracker hosted
  lane for its canonical proof. The correct Cloud Hypervisor operator workflow
  is an explicit temporary config mutation, not silently repurposing the sample.
- `keel story record` can drift inline proof references when a story has
  multiple command-based acceptance criteria. It is worth re-reading the story
  file after recording evidence and before submit.

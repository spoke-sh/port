---
created_at: 2026-04-11T23:18:11
---

# Reflection - Model Machine Runtime Class Contracts For Builder Lanes

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### VGYSoYxg0: Title
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

The contract stayed bounded once the builder lane was expressed as model
metadata instead of a new runtime verb.

Reusing the sample `demo` machine for tests initially conflicted with the local
K3s fixture because that cluster still requires a writable guest rootfs. The
runtime-class tests now clear the cluster fixture first so builder-lane
validation is exercised independently from cluster bootstrap rules.

---
created_at: 2026-03-06T16:50:05
---

# Reflection - Make Help Examples Runtime Agnostic

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vyfrp000: Title
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

The wording change was smaller than the blast radius of the user confusion.
`port --help` had been corrected once already, but README sections still
carried `nix develop` in artifact and development examples, which made the
runtime contract inconsistent again.

Keeping the canonical prerequisite boundary centered on required host tools and
`port doctor` produced a clearer operator story without changing runtime
behavior. The remaining transport failures are real runtime gaps and need their
own story rather than more help-surface explanation.

---
created_at: 2026-04-02T21:15:22
---

# Reflection - Rewrite Foundational AWS PVM Production Narrative

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### VFhLACP3l: Title
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

- The highest-value change was not adding more AWS text. It was establishing
  `docs/aws.md` as the canonical deployment narrative and turning `hosted`,
  `cloud`, and `pvm` into supporting contracts instead of competing summaries.
- The biggest source of drift was tense and posture: several docs still talked
  as if the hosted control plane was purely future work even though the repo
  now has live hosted machine and guest proofs. Tightening those statements was
  necessary to make the production story believable.
- Public MDX docs had to be updated in the same slice as the foundational
  docs. Leaving the site on the old hosted-standard framing would have recreated
  the same contradiction we were trying to remove.

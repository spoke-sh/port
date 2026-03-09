---
created_at: 2026-03-09T09:02:34
---

# Reflection - Implement Hosted Pvm Node Preparation

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzd3y000: Title
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

The control-plane preparation path was already strong inside the hosted runtime,
but the client still derived placement from static config before it ever spoke to
the live control plane. The key fix was to treat imported hosted inventory as a
first-class local overlay for doctor and preflight summary resolution, so a
successful `prepare-pvm-node` command immediately changes the canonical CLI
behavior instead of only changing server-side state.

The other practical lesson was that board verification comments must stay
current with real test names. `keel verify run` caught the stale command filters
even after the code and broader crate suites were green, which kept the story
evidence honest.

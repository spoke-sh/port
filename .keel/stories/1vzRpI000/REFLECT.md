---
created_at: 2026-03-08T21:50:28
---

# Reflection - Publish Hosted Service And Sandbox Workflow

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzSZY000: Title
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

- The implementation was already shipped in the runtime layer, but the public
  surfaces were inconsistent: CLI help and several canonical docs still
  described hosted service execution as follow-on work. The story value was in
  aligning the operator contract rather than changing runtime behavior.
- `keel story record` again wrote the wrong inline proof file for AC-01 after
  recording AC-02. The story README had to be corrected manually before
  commit. This is now a recurring board-tooling issue worth treating as a
  process constraint during submission.
- Grep-based proof scripts need to use fixed-string fragments instead of
  single-line full sentences because the canonical docs wrap long sentences
  across Markdown lines. Shorter fixed checks made the story verification
  stable.

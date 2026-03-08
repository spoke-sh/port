---
created_at: 2026-03-08T11:54:58
---

# Reflection - Publish Pvm Admission Workflow

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzJHG000: Title
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

- Publishing the workflow was not just a docs pass. Running the proof scripts
  surfaced two real regressions that the docs would have amplified instead of
  clarified: the `native` architecture alias was not being canonicalized during
  artifact selection, and the generated guest `/init` script was not loading
  the protection-mode marker.
- The strongest workflow proof combined three layers: CLI help and docs for
  discoverability, hosted PVM placement denial for admission semantics, and a
  real local standard-lane launch/stop cycle for behavioral preservation.
- Anchored config rewrites matter in repo-local proof scripts once example
  files grow explanatory comments. Loose regex replacements can silently target
  commented examples instead of the real machine stanza.

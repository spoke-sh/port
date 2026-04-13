---
created_at: 2026-04-13T07:01:14
---

# Reflection - Upgrade Keel Flake Input

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### VGgBuuaby: Title
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

The upgrade stayed scoped to `flake.lock`; `flake.nix` and the dependent
follow-edges for `atxt` and `paddles` did not need changes.

`nix flake lock --update-input keel` produced only the expected lockfile delta,
and `nix build .#keel --no-link` confirmed the exported package still builds as
`keel 0.2.1` after the bump.

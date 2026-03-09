---
created_at: 2026-03-09T01:54:05
---

# Reflection - Define Hosted Artifact Backend Contract

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzWNJ000: Title
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

Deterministic hosted artifact resolution fit cleanly once the control-plane
identity lookup moved into `port-model` instead of being inferred ad hoc in
runtime helpers. That kept the store-path contract reusable for later hosted
push and pull route stories.

The upgraded `keel` workflow is stricter than the earlier slices: acceptance
criteria need inline verify annotations before `keel story record` works, and
`keel story submit` now hard-fails on unchecked AC boxes and unresolved
reflection scaffold text. Future story setup should add those verify stubs
before implementation to avoid a second hygiene pass at the end.

The full workspace verification also exposed a stale CLI help assertion outside
the story scope. Fixing that immediately was the right move because it kept the
repo-level `cargo test -q` proof meaningful instead of relying only on targeted
crate tests.

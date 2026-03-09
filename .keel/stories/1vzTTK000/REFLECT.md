---
created_at: 2026-03-08T23:57:13
---

# Reflection - Surface Registered Placement Through Machine Commands

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzUYD000: Title
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

- Stored machine placement must become the canonical routing input after launch;
  probing live candidate nodes during later machine commands silently rewrites
  operator-visible placement and hides stale-registration failures.
- The hosted control plane needs to refresh placement state from disk on demand
  so canonical routing survives restarts and repaired runtime state files.
- Hosted tests that exercise the shared `.port/hosted/demo` control-plane layout
  need to hold the shared hosted server lock for the full test lifetime or they
  will race under full-suite execution even when individual tests pass.

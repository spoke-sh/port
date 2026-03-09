---
created_at: 2026-03-08T21:44:07
---

# Reflection - Route Hosted Service Lifecycle Through Live Runtime

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzSTP000: Title
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

- Reusing the existing hosted control-plane and node-agent route family kept
  the live service slice coherent. The only new external behavior was that the
  existing `services` and `secrets` endpoints now do real work instead of
  mutating local state.
- The clean split was public hosted wrappers plus `*_local` helpers. That let
  the CLI/runtime surface route through HTTP while the node agent still used
  the same runtime primitives without recursive control-plane calls.
- A single end-to-end CLI test with real control-plane, node-agent, and
  guest-agent fixtures caught the operator-facing regressions immediately once
  hosted `service secret` stopped being a local-runtime shortcut.

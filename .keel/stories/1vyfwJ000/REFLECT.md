---
created_at: 2026-03-06T17:00:13
---

# Reflection - Stabilize Runtime State For Guest Transport

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vyg1d000: Title
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

The runtime failure reports from the user pointed to two different problems:
stale launch-state cleanup and the missing live guest transport. Handling them
in one groundwork story kept the operator path usable while preserving a clean
follow-on transport story.

The strongest proof here was the real CLI relaunch under a fresh runtime root.
The unit tests cover the specific branches, but the launch/relaunch command
sequence exposed that the stale vsock file was the actual operator-facing
regression.

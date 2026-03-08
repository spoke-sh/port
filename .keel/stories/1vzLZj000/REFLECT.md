---
created_at: 2026-03-08T14:51:30
---

# Reflection - Implement Avf Local Machine Driver

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzM26000: Title
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

- The smallest workable AVF driver on a Linux development host is a real
  machine-driver path plus a launcher hook, not fake in-memory status. That let
  Port exercise canonical manifest, pid, status, and stop behavior without
  pretending Linux can execute Apple APIs directly.
- Generic process-existence checks were not enough for AVF stop handling.
  Killed launcher processes can remain as zombies briefly, so the AVF liveness
  path had to key off `/proc/<pid>/cmdline` presence, matching the more precise
  Firecracker liveness semantics.
- The existing runtime paths and manifest naming are still Firecracker-flavored.
  AVF-specific runtime metadata kept this story honest, but a later substrate
  neutralization pass should rename those canonical path and field labels.

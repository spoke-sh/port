---
created_at: 2026-03-08T15:58:11
---

# Reflection - Implement Streamed Pty And Log Follow

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzN4d000: Title
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

- PTY and log-follow needed an explicit framed stream layer on top of the
  existing guest socket. The earlier `Accepted` handshake alone was not enough
  because there was no deterministic way to distinguish streamed payload bytes
  from terminal completion state on the same direction of the connection.
- The streamed CLI behavior and the request/response compatibility path can
  coexist cleanly if the runtime owns both: `execute_guest_operation` can
  aggregate stream frames for legacy callers while `stream_guest_pty` and
  `stream_guest_logs` surface incremental output for the operator-facing CLI.
- The hosted test harness needed stronger readiness checks again once the suite
  started running more HTTP-backed tests in parallel. Retrying bind races and
  probing authenticated HTTP routes was the difference between stable workspace
  gates and misleading intermittent failures.

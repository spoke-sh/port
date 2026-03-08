---
created_at: 2026-03-07T21:21:47
---

# Reflection - Add Detached And Unix-Socket Forwarding

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vz6aJ000: Title
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

- Detached forwarding did not require a second public command tree. The
  workable cut was to keep `port guest forward` canonical and hide the daemon
  management mechanics behind an internal subcommand plus runtime manifests.
- Unix-socket forwarding fits cleanly into the existing forward protocol when
  the `listen` and `target` strings are treated as typed endpoint specs rather
  than as TCP-only addresses.
- The hosted guest-runtime slice made this follow-on much simpler: detached and
  Unix forwarding could build on the same hosted node-runtime resolution
  instead of inventing a different hosted forwarding transport.

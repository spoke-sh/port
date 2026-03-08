---
created_at: 2026-03-08T06:43:34
---

# Reflection - Publish Hosted SDK And API Clients

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzEPu000: Title
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

- The right first SDK surface was typed request construction, not a pretend
  live transport. That lets Port publish one supported machine/guest/service
  client model now without lying about the remote control plane boundary.
- Reusing `port-agent-protocol` request payloads in `port-sdk` keeps the guest
  client surface aligned with the canonical CLI and avoids inventing a second
  guest wire vocabulary.
- The docs needed the same precision as the code: once the SDK ships, help
  text and hosted docs can no longer call it future work. What remains planned
  now is transport, retries, response decoding, and advanced auth or tenancy
  features on top of the published request paths.

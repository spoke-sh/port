---
created_at: 2026-03-09T03:39:11
---

# Reflection - Publish Hosted Standard Cloud Workflow

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzY11000: Title
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

- The product change itself was mostly alignment work: once the hosted standard launch path shipped, the real task was bringing README, cloud docs, hosted docs, and CLI help into the same canonical story so operators were not taught a stale denial flow.
- The verification friction came from board metadata, not product behavior. `keel verify run` was sensitive to working-directory and shell-command shape, so the stable fix was to promote the docs/help verification into a repo-local script with an explicit repo-root wrapper instead of embedding a long fragile pipeline directly in the story annotation.
- Keeping an automated repo-local proof command in the published workflow materially improved the operator story. It gives a fast way to confirm the hosted standard lane is still documented and wired correctly without manually replaying the full control-plane and node-agent setup.

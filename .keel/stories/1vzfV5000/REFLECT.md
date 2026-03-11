---
created_at: 2026-03-11T11:44:26
---

# Reflection - Publish Service Reliability Operator Workflow

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### VDaNriX6O: Title
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

- A repository-local hosted proof script is practical for service workflows
  because the existing control-plane, node-agent, and guest-agent binaries can
  be started against a temporary runtime root without depending on real cloud
  infrastructure.
- The most brittle part of this story was Keel evidence recording rather than
  the workflow itself: command proofs that invoke `bash -lc` can hang or mislink
  proof files, so the story README and evidence logs needed manual repair after
  the successful proof runs.
- Publishing the same workflow in README, hosted docs, CLI help, and sample
  config made the shipped restart/health/secret contract discoverable from the
  surfaces operators actually inspect first.

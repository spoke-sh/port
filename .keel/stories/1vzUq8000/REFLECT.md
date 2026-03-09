---
created_at: 2026-03-09T01:20:05
---

# Reflection - Surface Durable Hosted Fleet State

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzVqP000: Title
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

- Surfacing hosted fleet state as a first-class `MachineStatus` field was more stable than trying to encode imported, stale, and missing-registration nodes into generic hosted status strings. The explicit fleet-node structure kept the runtime merge logic and CLI render logic aligned.
- Hosted control-plane tests can pick up durable registration state from `.port/hosted/<control-plane>` if they reuse the same control-plane name. Cleaning registered state before and after merge-failure tests is necessary to avoid cross-test contamination.
- CLI unit tests were a better proof surface for the operator-facing render contract than a full spawned control-plane integration test. The integration harness was useful for exploration, but the stable acceptance proofs came from runtime state tests plus pure render tests.

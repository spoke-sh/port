---
created_at: 2026-03-09T09:52:56
---

# Reflection - Implement Local Cloud Hypervisor Machine Driver

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzdqi000: Title
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

- Reusing the existing machine-driver seam kept the Cloud Hypervisor slice small: launch, status, stop, monitor, and top could be added without disturbing the hosted control-plane path.
- The runtime status contract needed Cloud Hypervisor-specific config and log paths, so the driver writes a sidecar `cloud-hypervisor-runtime.json` instead of overloading the Firecracker-named runtime files.
- The first failing test exposed that the fake hypervisor process should be treated as a live pid by existence for Cloud Hypervisor and AVF runtime ownership; process-existence probing is the correct contract here because there is no stable machine-id CLI flag like Firecracker's `--id`.

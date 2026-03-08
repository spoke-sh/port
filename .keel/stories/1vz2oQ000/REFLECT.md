---
created_at: 2026-03-07T17:32:52
---

# Reflection - Model Substrates And Protection Modes

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vz30m000: Title
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

### 1vz3A9000: Separate Architecture From Protection-Mode Support
| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When Port expands from one Linux Firecracker lane into PVM, AVF, and additional substrate lanes |
| **Insight** | General architecture support and protected-virtualization support cannot share one boolean or one token. Port needs explicit substrate, protection-mode, and architecture fields so it can say “arm64 exists” without implying “arm64 PVM is shipped.” |
| **Suggested Action** | Keep machine compatibility validation and docs keyed on substrate plus protection mode plus resolved architecture, and reject unsupported combinations explicitly. |
| **Applies To** | crates/port-model/**, crates/port-runtime/**, docs/**, examples/port.toml |
| **Observed At** | 2026-03-08T01:32:00Z |
| **Score** | 0.93 |
| **Confidence** | 0.95 |
| **Applied** | yes |

## Observations

- The new model landed cleanly once the scope stayed narrow: machine-level
  substrate/protection/architecture plus artifact compatibility metadata was
  enough to express the new support matrix without prematurely introducing a
  full runtime-driver abstraction.
- Runtime diagnostics were the right first enforcement point. Adding lane-aware
  checks to `port doctor` and launch-time contract validation gives operators
  actionable failures before later lifecycle or hosted work lands.
- Documentation drift was a real risk. README and `docs/cloud.md` still carried
  the old “PVM dropped” story, so this slice had to update the docs in the same
  change or the board would have contradicted the implementation direction.
- `keel story record --cmd` appears to leave long-lived recorder processes in
  this environment. Recording proof through generated log files plus annotated
  acceptance criteria was reliable and should be reused if the direct command
  path keeps hanging.

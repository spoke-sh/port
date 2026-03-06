---
created_at: 2026-03-06T15:52:06
---

# Reflection - Model Cloud Linux Providers

## Knowledge

### 1vyezc000: Parse-Test Canonical Example Configs
| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | When `examples/port.toml` becomes a canonical CLI proof surface for new provider or platform lanes. |
| **Insight** | String-matching example config content is not enough once the example carries workflow-critical provider identity; a parse test catches drift between the checked-in example and the shared model. |
| **Suggested Action** | Add a `PortConfig::from_path` test for canonical example files whenever model shape changes. |
| **Applies To** | `examples/*.toml`, `crates/port-model/src/lib.rs` |
| **Linked Knowledge IDs** | |
| **Observed At** | 2026-03-06T23:53:00Z |
| **Score** | 0.77 |
| **Confidence** | 0.94 |
| **Applied** | yes |

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vyexi000: Title
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

- Adding provider identity at `HostSpec` was the right seam because later CLI/runtime diagnostics can now branch on declared intent instead of inferring cloud meaning from `connection.mode = ssh`.
- Extending the canonical example config with remote provider hosts and machine stubs gives later cloud stories stable names to verify against without disturbing the existing local `demo` workflow.
- Using an isolated `CARGO_TARGET_DIR` kept verification from mutating the accidentally tracked repo-local `target/` tree while the MVP work continues.

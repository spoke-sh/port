---
created_at: 2026-03-08T09:13:28
---

# Reflection - Publish Hosted Demo Workflow And Evidence

## Knowledge

### 1w03mg000: Prefer Reusable Demo Scripts Over Test-Only Proof For Operator Workflows
| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | A story's acceptance depends on operators being able to discover and reproduce a workflow, not only on internal integration tests passing. |
| **Insight** | A small repo-local demo script is higher-signal than a test name for operator-facing evidence because it can be linked from docs, called by verification scripts, and run directly by humans without understanding the test harness. |
| **Suggested Action** | When a story is primarily about workflow discoverability or reproducibility, publish a reusable demo script and have the `keel` verification script call it instead of recording only crate-test commands. |
| **Applies To** | `scripts/*.sh`, `.keel/stories/*/verify-ac-*.sh`, operator workflow docs |
| **Linked Knowledge IDs** | 1w03m1000 |
| **Observed At** | 2026-03-08T09:16:00Z |
| **Score** | 0.84 |
| **Confidence** | 0.9 |
| **Applied** | yes |

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzGky000: Title
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

The story was smaller than the transport cutover, but it still benefitted from the same verification discipline. The new reusable `scripts/hosted-demo.sh` proof made the documentation tighter because every step had to be runnable outside the test harness.

What went well was using the same deliberate split between server config and client config from the previous story. That let the demo evidence prove the live HTTP path again instead of quietly regressing to runtime-root coupling.

The main difficulty was keeping the operator docs honest about the limits that still remain. The demo is real and reproducible, but it is still explicitly a single-node repository-local lane, with hosted `copy` depending on node-visible host paths and hosted `forward` still relying on the repo-local listener lifecycle.

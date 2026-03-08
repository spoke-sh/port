---
created_at: 2026-03-08T09:08:00
---

# Reflection - Route Hosted CLI And SDK Through Live Transport

## Knowledge

### 1w03m0000: Preserve Hosted Control Metadata After Node-Local Execution
| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Hosted control-plane routes call the node agent, and the node agent localizes the machine host connection to `Local` before reusing runtime helpers. |
| **Insight** | Reusing local runtime helpers inside the node agent will silently downgrade `MachineStatus`, `StopResult`, `MachineMonitorReport`, and `MachineTopReport` back to the local control contract unless the hosted route metadata is re-applied before the HTTP response is encoded. |
| **Suggested Action** | Keep a projection layer at the node-agent boundary that restores hosted control contracts and route context after any localized runtime call. |
| **Applies To** | `crates/port-runtime/src/hosted_control_plane.rs`, hosted lifecycle and monitoring handlers |
| **Linked Knowledge IDs** | |
| **Observed At** | 2026-03-08T09:10:00Z |
| **Score** | 0.93 |
| **Confidence** | 0.96 |
| **Applied** | yes |

### 1w03m1000: Prove Transport Cutovers With Divergent Client And Server Configs
| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | A transport cutover can appear to work when the client and server share the same local runtime root, even if the client still bypasses the network path. |
| **Insight** | The most reliable regression test for this class of change is to boot the long-lived server processes with the correct runtime root, then run the CLI or SDK against a second config whose hosted runtime root is intentionally wrong. |
| **Suggested Action** | Use split server/client configs in future hosted transport tests whenever the goal is to prove that the network path, not a local shortcut, is carrying the request. |
| **Applies To** | `crates/port-cli/tests/*hosted*`, `crates/port-runtime/src/lib.rs` hosted tests, future hosted SDK tests |
| **Linked Knowledge IDs** | |
| **Observed At** | 2026-03-08T09:12:00Z |
| **Score** | 0.9 |
| **Confidence** | 0.95 |
| **Applied** | yes |

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vzGfg000: Title
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

The cutover itself was straightforward once the live hosted client existed in `port-sdk`, but the verification gap was real: several tests were still proving the old config-backed shortcut instead of the transport the story claimed to ship.

The most important defect I found during the wider verification pass was in the node-agent response layer. The HTTP transport worked, but the node agent reused localized runtime helpers and returned local control metadata until a projection layer re-applied the hosted control contract and route context. That would have been easy to miss without the broadened runtime and CLI verification.

The remaining limits are explicit now instead of being hidden by stale docs. Hosted `machine` plus hosted `guest exec|copy|pty|logs` use the live HTTP path, while hosted `copy` still assumes node-visible host paths in the single-node demo and hosted `forward` still depends on the repo-local listener lifecycle.

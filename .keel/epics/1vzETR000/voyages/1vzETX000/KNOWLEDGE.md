---
created_at: 2026-03-08T09:14:24
---

# Knowledge - 1vzETX000

> Automated synthesis of story reflections.

## Story Knowledge

## Story: Publish Hosted Demo Workflow And Evidence (1vzEVX000)

### 1w03mg000: Prefer Reusable Demo Scripts Over Test-Only Proof For Operator Workflows

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | A story's acceptance depends on operators being able to discover and reproduce a workflow, not only on internal integration tests passing. |
| **Insight** | A small repo-local demo script is higher-signal than a test name for operator-facing evidence because it can be linked from docs, called by verification scripts, and run directly by humans without understanding the test harness. |
| **Suggested Action** | When a story is primarily about workflow discoverability or reproducibility, publish a reusable demo script and have the `keel` verification script call it instead of recording only crate-test commands. |
| **Applies To** | `scripts/*.sh`, `.keel/stories/*/verify-ac-*.sh`, operator workflow docs |
| **Applied** | yes |



---

## Story: Route Hosted CLI And SDK Through Live Transport (1vzEVo000)

### 1w03m0000: Preserve Hosted Control Metadata After Node-Local Execution

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Hosted control-plane routes call the node agent, and the node agent localizes the machine host connection to `Local` before reusing runtime helpers. |
| **Insight** | Reusing local runtime helpers inside the node agent will silently downgrade `MachineStatus`, `StopResult`, `MachineMonitorReport`, and `MachineTopReport` back to the local control contract unless the hosted route metadata is re-applied before the HTTP response is encoded. |
| **Suggested Action** | Keep a projection layer at the node-agent boundary that restores hosted control contracts and route context after any localized runtime call. |
| **Applies To** | `crates/port-runtime/src/hosted_control_plane.rs`, hosted lifecycle and monitoring handlers |
| **Applied** | yes |

### 1w03m1000: Prove Transport Cutovers With Divergent Client And Server Configs

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | A transport cutover can appear to work when the client and server share the same local runtime root, even if the client still bypasses the network path. |
| **Insight** | The most reliable regression test for this class of change is to boot the long-lived server processes with the correct runtime root, then run the CLI or SDK against a second config whose hosted runtime root is intentionally wrong. |
| **Suggested Action** | Use split server/client configs in future hosted transport tests whenever the goal is to prove that the network path, not a local shortcut, is carrying the request. |
| **Applies To** | `crates/port-cli/tests/*hosted*`, `crates/port-runtime/src/lib.rs` hosted tests, future hosted SDK tests |
| **Applied** | yes |



---

## Synthesis

### Gyt82vp7v: Prefer Reusable Demo Scripts Over Test-Only Proof For Operator Workflows

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | A story's acceptance depends on operators being able to discover and reproduce a workflow, not only on internal integration tests passing. |
| **Insight** | A small repo-local demo script is higher-signal than a test name for operator-facing evidence because it can be linked from docs, called by verification scripts, and run directly by humans without understanding the test harness. |
| **Suggested Action** | When a story is primarily about workflow discoverability or reproducibility, publish a reusable demo script and have the `keel` verification script call it instead of recording only crate-test commands. |
| **Applies To** | `scripts/*.sh`, `.keel/stories/*/verify-ac-*.sh`, operator workflow docs |
| **Linked Knowledge IDs** | 1w03mg000 |
| **Score** | 0.84 |
| **Confidence** | 0.90 |
| **Applied** | yes |

### 4o4SSAlZb: Preserve Hosted Control Metadata After Node-Local Execution

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Hosted control-plane routes call the node agent, and the node agent localizes the machine host connection to `Local` before reusing runtime helpers. |
| **Insight** | Reusing local runtime helpers inside the node agent will silently downgrade `MachineStatus`, `StopResult`, `MachineMonitorReport`, and `MachineTopReport` back to the local control contract unless the hosted route metadata is re-applied before the HTTP response is encoded. |
| **Suggested Action** | Keep a projection layer at the node-agent boundary that restores hosted control contracts and route context after any localized runtime call. |
| **Applies To** | `crates/port-runtime/src/hosted_control_plane.rs`, hosted lifecycle and monitoring handlers |
| **Linked Knowledge IDs** | 1w03m0000 |
| **Score** | 0.93 |
| **Confidence** | 0.96 |
| **Applied** | yes |

### tkPtuC6BJ: Prove Transport Cutovers With Divergent Client And Server Configs

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | A transport cutover can appear to work when the client and server share the same local runtime root, even if the client still bypasses the network path. |
| **Insight** | The most reliable regression test for this class of change is to boot the long-lived server processes with the correct runtime root, then run the CLI or SDK against a second config whose hosted runtime root is intentionally wrong. |
| **Suggested Action** | Use split server/client configs in future hosted transport tests whenever the goal is to prove that the network path, not a local shortcut, is carrying the request. |
| **Applies To** | `crates/port-cli/tests/*hosted*`, `crates/port-runtime/src/lib.rs` hosted tests, future hosted SDK tests |
| **Linked Knowledge IDs** | 1w03m1000 |
| **Score** | 0.90 |
| **Confidence** | 0.95 |
| **Applied** | yes |


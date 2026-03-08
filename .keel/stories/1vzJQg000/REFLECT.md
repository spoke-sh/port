---
created_at: 2026-03-08T12:20:57
---

# Reflection - Define Prepared Pvm Host Kit Contract

## Knowledge

### 1vzJQg000: Share PVM Host-Kit Contracts Across Local And Hosted Lanes
| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When Port needs to reason about Firecracker/PVM readiness before a real hosted launch path exists |
| **Insight** | Modeling only hosted PVM state (`planned` or `ready`) is not enough; the hosted node inventory must carry the same host-kit contract shape as the local lane so doctor, placement, and later node-agent launch can reuse one source of truth. |
| **Suggested Action** | Add new PVM launch or placement work against the shared `PvmHostKit` contract first, then build runtime behavior on top of it. |
| **Applies To** | `crates/port-model`, `crates/port-runtime`, hosted inventory and doctor surfaces |
| **Linked Knowledge IDs** | |
| **Observed At** | 2026-03-08T12:21:30Z |
| **Score** | 0.84 |
| **Confidence** | 0.96 |
| **Applied** | yes |

## Observations

Adding the host-kit contract to both `FirecrackerPvmLaneContract` and
`HostedPvmCapability` kept this slice small and future-proofed the next hosted
launch story. The main surprise was that malformed configs are rejected by the
CLI load/validation path before `doctor` can run, which is correct, but it
meant the AC-02 proof needed to focus on explicit config-load and launch
failure rather than trying to force `doctor` through an invalid config.

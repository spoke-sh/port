---
created_at: 2026-03-08T14:19:36
---

# Knowledge - 1vzJP2000

> Automated synthesis of story reflections.

## Story Knowledge

## Story: Define Prepared Pvm Host Kit Contract (1vzJQg000)

### 1vzJQg000: Share PVM Host-Kit Contracts Across Local And Hosted Lanes

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When Port needs to reason about Firecracker/PVM readiness before a real hosted launch path exists |
| **Insight** | Modeling only hosted PVM state (`planned` or `ready`) is not enough; the hosted node inventory must carry the same host-kit contract shape as the local lane so doctor, placement, and later node-agent launch can reuse one source of truth. |
| **Suggested Action** | Add new PVM launch or placement work against the shared `PvmHostKit` contract first, then build runtime behavior on top of it. |
| **Applies To** | `crates/port-model`, `crates/port-runtime`, hosted inventory and doctor surfaces |
| **Applied** | yes |



---

## Story: Implement Node Agent Pvm Launch Path (1vzJSi000)

### 1vzJSi000: Localize Hosted Node Launch Down To One Machine And Host

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When a hosted node agent reuses the shared local launcher for one machine request |
| **Insight** | Rewriting only the host connection to `Local` is not enough. The localized config must be narrowed to the target machine and host, and hosted-only inventory must be removed, or config validation will fail on stale hosted references before the launch path runs. |
| **Suggested Action** | Keep node-agent launch localization as a deliberate scope-reduction step before calling shared local runtime helpers. |
| **Applies To** | `crates/port-runtime/src/hosted_control_plane.rs`, hosted-to-local runtime adaptation paths |
| **Applied** | yes |



---

## Synthesis

### uvMOMCTI9: Share PVM Host-Kit Contracts Across Local And Hosted Lanes

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When Port needs to reason about Firecracker/PVM readiness before a real hosted launch path exists |
| **Insight** | Modeling only hosted PVM state (`planned` or `ready`) is not enough; the hosted node inventory must carry the same host-kit contract shape as the local lane so doctor, placement, and later node-agent launch can reuse one source of truth. |
| **Suggested Action** | Add new PVM launch or placement work against the shared `PvmHostKit` contract first, then build runtime behavior on top of it. |
| **Applies To** | `crates/port-model`, `crates/port-runtime`, hosted inventory and doctor surfaces |
| **Linked Knowledge IDs** | 1vzJQg000 |
| **Score** | 0.84 |
| **Confidence** | 0.96 |
| **Applied** | yes |

### LBGfsRJgU: Localize Hosted Node Launch Down To One Machine And Host

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When a hosted node agent reuses the shared local launcher for one machine request |
| **Insight** | Rewriting only the host connection to `Local` is not enough. The localized config must be narrowed to the target machine and host, and hosted-only inventory must be removed, or config validation will fail on stale hosted references before the launch path runs. |
| **Suggested Action** | Keep node-agent launch localization as a deliberate scope-reduction step before calling shared local runtime helpers. |
| **Applies To** | `crates/port-runtime/src/hosted_control_plane.rs`, hosted-to-local runtime adaptation paths |
| **Linked Knowledge IDs** | 1vzJSi000 |
| **Score** | 0.82 |
| **Confidence** | 0.95 |
| **Applied** | yes |


---
source_type: Story
source: stories/1vzJQg000/REFLECT.md
scope: 1vzJKE000/1vzJP2000
source_story_id: 1vzJQg000
created_at: 2026-03-08T12:20:57
---

### 1vzJQg000: Share PVM Host-Kit Contracts Across Local And Hosted Lanes

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When Port needs to reason about Firecracker/PVM readiness before a real hosted launch path exists |
| **Insight** | Modeling only hosted PVM state (`planned` or `ready`) is not enough; the hosted node inventory must carry the same host-kit contract shape as the local lane so doctor, placement, and later node-agent launch can reuse one source of truth. |
| **Suggested Action** | Add new PVM launch or placement work against the shared `PvmHostKit` contract first, then build runtime behavior on top of it. |
| **Applies To** | `crates/port-model`, `crates/port-runtime`, hosted inventory and doctor surfaces |
| **Linked Knowledge IDs** |  |
| **Observed At** | 2026-03-08T12:21:30+00:00 |
| **Score** | 0.84 |
| **Confidence** | 0.96 |
| **Applied** | yes |

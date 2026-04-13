---
source_type: Story
source: stories/VGcgtDfDT/REFLECT.md
scope: VGcgU9T57/VGcghwZrb
source_story_id: VGcgtDfDT
created_at: 2026-04-12T17:35:58
---

### VGct0mwbD: Hosted Proof Harnesses Need Isolated Control-Plane State

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | When a proof script starts a hosted control plane and node agents inside this repository workspace |
| **Insight** | Hosted proof harnesses can collide with existing `.port/hosted/<control-plane>` state and stale binary assumptions unless they use a unique temporary control-plane name and resolve the CLI through the active `CARGO_TARGET_DIR`. |
| **Suggested Action** | Give each hosted proof run a unique control-plane name and derive the CLI binary path from `CARGO_TARGET_DIR` before starting long-lived harness processes. |
| **Applies To** | `scripts/render-hosted-*.sh` |
| **Linked Knowledge IDs** |  |
| **Observed At** | 2026-04-13T00:35:58+00:00 |
| **Score** | 0.86 |
| **Confidence** | 0.92 |
| **Applied** | yes |

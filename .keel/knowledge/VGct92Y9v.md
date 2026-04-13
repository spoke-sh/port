---
source_type: Story
source: stories/VGcgtFI9v/REFLECT.md
scope: VGcgU9T57/VGcghwZrb
source_story_id: VGcgtFI9v
created_at: 2026-04-12T17:44:58
---

### VGct92Y9v: Hosted Proof Harnesses Must Seed Registrations Before Control-Plane Start

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | When a hosted proof script seeds `registered-nodes.json` directly instead of depending on live registration refresh during the run |
| **Insight** | `control-plane serve` loads registered node state into memory at startup, so proof harnesses that hand-author registration state must write it before starting the control plane or the route resolver will treat every candidate node as missing. |
| **Suggested Action** | Reserve node bind addresses first, persist registered-node state with current freshness timestamps, then start `control-plane serve` and the node-agent processes. |
| **Applies To** | `scripts/render-hosted-*.sh`, hosted control-plane proof harnesses |
| **Linked Knowledge IDs** |  |
| **Observed At** | 2026-04-13T00:44:00+00:00 |
| **Score** | 0.89 |
| **Confidence** | 0.95 |
| **Applied** | yes |

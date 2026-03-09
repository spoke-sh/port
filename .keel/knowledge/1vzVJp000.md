---
source_type: Story
source: stories/1vzUq6000/REFLECT.md
scope: 1vzUnI000/1vzUoK000
source_story_id: 1vzUq6000
created_at: 2026-03-09T00:39:32
---

### 1vzVJp000: Isolate Default Hosted Test State From Demo Runtime Files

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | When hosted control-plane tests use the default `demo` control plane and static node bindings. |
| **Insight** | Persisted hosted registry and placement files under `.port/hosted/demo` can override static test bindings because registered nodes are resolved before static bindings. Isolated reruns then fail differently from full suites depending on leftover on-disk state. |
| **Suggested Action** | Clear default hosted runtime state in test helpers before starting a static-bound control plane, or use unique control-plane names per test when persisted state is part of the scenario. |
| **Applies To** | `crates/port-runtime/src/hosted_control_plane.rs`, hosted control-plane tests |
| **Linked Knowledge IDs** |  |
| **Observed At** | 2026-03-09T00:40:00+00:00 |
| **Score** | 0.86 |
| **Confidence** | 0.96 |
| **Applied** | yes |

---
source_type: Story
source: stories/1vyerj000/REFLECT.md
scope: 1vydg7000/1vyeq5000
source_story_id: 1vyerj000
created_at: 2026-03-06T15:52:06
---

### 1vyezc000: Parse-Test Canonical Example Configs

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | When `examples/port.toml` becomes a canonical CLI proof surface for new provider or platform lanes. |
| **Insight** | String-matching example config content is not enough once the example carries workflow-critical provider identity; a parse test catches drift between the checked-in example and the shared model. |
| **Suggested Action** | Add a `PortConfig::from_path` test for canonical example files whenever model shape changes. |
| **Applies To** | `examples/*.toml`, `crates/port-model/src/lib.rs` |
| **Linked Knowledge IDs** |  |
| **Observed At** | 2026-03-06T23:53:00+00:00 |
| **Score** | 0.77 |
| **Confidence** | 0.94 |
| **Applied** | yes |

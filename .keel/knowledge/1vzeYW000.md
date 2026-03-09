---
source_type: Story
source: stories/1vzeYW000/REFLECT.md
scope: 1vzW8e000/1vzeWr000
source_story_id: 1vzeYW000
created_at: 2026-03-09T11:15:00
---

### 1vzeYW000: Keep OCI pull on the same artifact path contract as every other backend

| Field | Value |
|-------|-------|
| **Category** | artifact-mobility |
| **Context** | When adding a remote pull backend that downloads artifact bytes before materializing them into the workspace-local artifact layout. |
| **Insight** | Pull backends should use backend-private staging and then hydrate the canonical cache and local artifact paths, otherwise each backend grows its own retrieval layout and the CLI stops being predictable. |
| **Suggested Action** | Keep staging directories internal to the adapter and add a path-parity proof whenever a new artifact distribution backend is introduced. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, `crates/port-cli/tests/artifact_commands.rs`, future artifact mobility stories |
| **Linked Knowledge IDs** |  |
| **Observed At** | 2026-03-09T19:15:00+00:00 |
| **Score** | 0.83 |
| **Confidence** | 0.95 |
| **Applied** | yes |

---
source_type: Story
source: stories/VFDk8gRoD/REFLECT.md
scope: VFDhlRjOf/VFDk8fdnG
source_story_id: VFDk8gRoD
created_at: 2026-03-28T22:03:14
---

### VFDmLq9xQ: Firecracker test doubles must preserve launch argv

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | Runtime and CLI tests that rely on local Firecracker machine-status detection |
| **Insight** | Port classifies local Firecracker processes by inspecting live argv for both `firecracker` and `--id <machine>`, so a fake helper that `exec`s into another binary can make a healthy test process look stale. |
| **Suggested Action** | Keep fake Firecracker helpers running under a command line that still includes the `firecracker` script path and launch args, or update the test double explicitly when machine-status matching changes. |
| **Applies To** | crates/port-runtime/src/lib.rs; crates/port-cli/tests/machine_commands.rs |
| **Linked Knowledge IDs** |  |
| **Observed At** | 2026-03-29T05:05:00+00:00 |
| **Score** | 0.74 |
| **Confidence** | 0.92 |
| **Applied** | yes |

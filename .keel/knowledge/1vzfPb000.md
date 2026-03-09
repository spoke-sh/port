---
source_type: Story
source: stories/1vzeYA000/REFLECT.md
scope: 1vzW8e000/1vzeWr000
source_story_id: 1vzeYA000
created_at: 2026-03-09T11:32:34
---

### 1vzfPb000: Keel Story Evidence Commands Need An Explicit Repo Root

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Recording story proof with `keel story record --cmd` for repo-scoped commands |
| **Insight** | `keel story record` does not guarantee execution from the repository root, so relative-path proof commands can fail even when the same command succeeds interactively from the shell |
| **Suggested Action** | Wrap repo-scoped proof commands in `bash -lc "cd /repo/root && ..."` or use absolute paths |
| **Applies To** | `.keel/stories/*`, `keel story record`, repo-local verification commands |
| **Linked Knowledge IDs** |  |
| **Observed At** | 2026-03-09T19:32:34+00:00 |
| **Score** | 0.67 |
| **Confidence** | 0.93 |
| **Applied** | AC-01 and AC-02 evidence for this story use an explicit `cd /home/alex/workspace/spoke-sh/port && ...` wrapper |

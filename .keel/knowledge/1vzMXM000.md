---
source_type: Story
source: stories/1vzMXM000/REFLECT.md
scope: 1vzMVF000/1vzMVY000
source_story_id: 1vzMXM000
created_at: 2026-03-08T19:18:17
---

### 1vzMXM000: Workflow-surface stories need proof that matches the published wording

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | When a story is mostly CLI/help/docs work instead of a deep runtime change. |
| **Insight** | Doc-only acceptance is still fragile unless the proof scripts check the exact published keywords and pair them with executable workflow tests. The fastest way to keep these stories honest was to combine `rg`-based surface checks with targeted CLI/runtime tests for the workflows named in the docs. |
| **Suggested Action** | For future workflow-surface stories, write verify scripts that inspect the text and replay the referenced commands before submit. |
| **Applies To** | `.keel/stories/*/verify-ac-*.sh`, CLI help text, README and docs updates |
| **Linked Knowledge IDs** |  |
| **Observed At** | 2026-03-08T19:19:00+00:00 |
| **Score** | 0.74 |
| **Confidence** | 0.94 |
| **Applied** | yes |

---
source_type: Story
source: stories/1vzQIq000/REFLECT.md
scope: 1vzETR000/1vzQEj000
source_story_id: 1vzQIq000
created_at: 2026-03-08T19:31:35
---

### 1vzQKh000: Keel Story Record Proof Mapping Can Drift Across Same-SRS ACs

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Recording command proof for stories whose acceptance criteria share one SRS requirement prefix |
| **Insight** | `keel story record` can overwrite the inline `proof:` annotation for an earlier AC with the later AC's evidence file, while leaving the checkbox state unchanged. |
| **Suggested Action** | Inspect the story README after every multi-AC `story record` run and correct proof links or checkboxes before submit. |
| **Applies To** | `.keel/stories/*/README.md`, `keel story record` workflows |
| **Linked Knowledge IDs** |  |
| **Observed At** | 2026-03-08T19:31:35+00:00 |
| **Score** | 0.84 |
| **Confidence** | 0.96 |
| **Applied** | yes |

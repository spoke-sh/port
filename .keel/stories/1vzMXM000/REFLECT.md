---
created_at: 2026-03-08T19:18:17
---

# Reflection - Publish Streamed Guest Workflow Surface

## Knowledge

### 1vzMXM000: Workflow-surface stories need proof that matches the published wording
| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | When a story is mostly CLI/help/docs work instead of a deep runtime change. |
| **Insight** | Doc-only acceptance is still fragile unless the proof scripts check the exact published keywords and pair them with executable workflow tests. The fastest way to keep these stories honest was to combine `rg`-based surface checks with targeted CLI/runtime tests for the workflows named in the docs. |
| **Suggested Action** | For future workflow-surface stories, write verify scripts that inspect the text and replay the referenced commands before submit. |
| **Applies To** | `.keel/stories/*/verify-ac-*.sh`, CLI help text, README and docs updates |
| **Linked Knowledge IDs** | |
| **Observed At** | 2026-03-08T19:19:00Z |
| **Score** | 0.74 |
| **Confidence** | 0.94 |
| **Applied** | yes |

## Observations

- The strongest proof for this slice came from keeping the docs and the tests coupled. The CLI help keyword guard plus the three verify scripts made it straightforward to see whether the published workflows still matched reality.
- The hosted forward behavior needed extra wording discipline. The capability is now live, but hosted detached lifecycle management is not, so the docs had to describe the boundary explicitly instead of inheriting the local forward wording.
- `keel story record` still mis-associates proof links when multiple acceptance criteria share the same SRS prefix. I had to correct the story README manually again before closing the slice.

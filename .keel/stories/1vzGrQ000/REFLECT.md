---
created_at: 2026-03-08T09:49:24
---

# Reflection - Publish PVM Operator Proof Workflow

## Knowledge

### 1w03x0000: Order Operator Proofs So The Log Tells The Story
| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | A repository-local proof script demonstrates several commands in sequence and the first failing-looking lines can dominate the acceptance review even when later commands show the intended end state. |
| **Insight** | Proof scripts should be ordered so the log explains the workflow clearly from top to bottom. Rebuilding prerequisite artifacts before a diagnostic step can make the evidence materially easier to review without changing the underlying behavior. |
| **Suggested Action** | When writing workflow proofs, read the first screenful of the resulting log and reorder the commands until that excerpt communicates the intended operator outcome. |
| **Applies To** | `.keel/stories/*/verify-ac-*.sh`, operator workflow evidence |
| **Linked Knowledge IDs** | 1w03mg000, 1w03w0000 |
| **Observed At** | 2026-03-08T09:50:00Z |
| **Score** | 0.74 |
| **Confidence** | 0.9 |
| **Applied** | yes |

## Observations

The docs and help surface were already partly prepared by the earlier PVM
stories, so the main work here was consolidating them into one coherent
operator path and then proving that path with rerunnable scripts.

The most useful cleanup was reordering the workflow proof. Starting with
`port doctor` before the standard artifacts were rebuilt made the excerpt look
more broken than the final state; rebuilding first made the evidence much
easier to review.

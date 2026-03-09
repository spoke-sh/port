---
created_at: 2026-03-08T19:31:35
---

# Reflection - Define Hosted Detached Forward Contract

## Knowledge

### 1vzQKh000: Keel Story Record Proof Mapping Can Drift Across Same-SRS ACs
| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Recording command proof for stories whose acceptance criteria share one SRS requirement prefix |
| **Insight** | `keel story record` can overwrite the inline `proof:` annotation for an earlier AC with the later AC's evidence file, while leaving the checkbox state unchanged. |
| **Suggested Action** | Inspect the story README after every multi-AC `story record` run and correct proof links or checkboxes before submit. |
| **Applies To** | `.keel/stories/*/README.md`, `keel story record` workflows |
| **Linked Knowledge IDs** | |
| **Observed At** | 2026-03-08T19:31:35Z |
| **Score** | 0.84 |
| **Confidence** | 0.96 |
| **Applied** | yes |

## Observations

The contract slice stayed narrow and that was the right call. Adding explicit
detached-forward routes plus `forward_name` to the shared hosted route context
was enough to unblock the later runtime stories without dragging the control
plane implementation into this commit.

The main surprise was that the upgraded `keel` verifier stayed correct while
`keel story record` still rewrote AC1's proof comment to `ac-2.log` and left
both AC checkboxes unchecked. Manual README inspection remains mandatory before
submit when multiple ACs map to the same SRS requirement.

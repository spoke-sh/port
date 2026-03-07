---
created_at: 2026-03-06T16:11:47
---

# Knowledge - 1vyfCm000

> Automated synthesis of story reflections.

## Story Knowledge

## Story: Fix Help Example Guidance (1vyfD0000)

### 1vyfFY000: Say When Help Examples Depend On Repo Context

| Field | Value |
|-------|-------|
| **Category** | documentation |
| **Context** | When CLI help includes commands that reference repo-relative config paths or environment-provided tooling. |
| **Insight** | Example commands that are syntactically correct can still feel broken if the help text does not state the repository-root assumption and the dependency environment right next to the examples. |
| **Suggested Action** | Put prerequisite and working-directory assumptions adjacent to CLI examples whenever commands depend on repo-relative files or external tools on PATH. |
| **Applies To** | `crates/port-cli/src/lib.rs`, CLI help text, operator docs |
| **Applied** | yes |



---

## Synthesis

### WnN3E9O4Q: Say When Help Examples Depend On Repo Context

| Field | Value |
|-------|-------|
| **Category** | documentation |
| **Context** | When CLI help includes commands that reference repo-relative config paths or environment-provided tooling. |
| **Insight** | Example commands that are syntactically correct can still feel broken if the help text does not state the repository-root assumption and the dependency environment right next to the examples. |
| **Suggested Action** | Put prerequisite and working-directory assumptions adjacent to CLI examples whenever commands depend on repo-relative files or external tools on PATH. |
| **Applies To** | `crates/port-cli/src/lib.rs`, CLI help text, operator docs |
| **Linked Knowledge IDs** | 1vyfFY000 |
| **Score** | 0.82 |
| **Confidence** | 0.96 |
| **Applied** | yes |


---
created_at: 2026-03-06T16:10:46
---

# Reflection - Fix Help Example Guidance

## Knowledge

### 1vyfFY000: Say When Help Examples Depend On Repo Context
| Field | Value |
|-------|-------|
| **Category** | documentation |
| **Context** | When CLI help includes commands that reference repo-relative config paths or environment-provided tooling. |
| **Insight** | Example commands that are syntactically correct can still feel broken if the help text does not state the repository-root assumption and the dependency environment right next to the examples. |
| **Suggested Action** | Put prerequisite and working-directory assumptions adjacent to CLI examples whenever commands depend on repo-relative files or external tools on PATH. |
| **Applies To** | `crates/port-cli/src/lib.rs`, CLI help text, operator docs |
| **Linked Knowledge IDs** | |
| **Observed At** | 2026-03-07T00:11:00Z |
| **Score** | 0.82 |
| **Confidence** | 0.96 |
| **Applied** | yes |

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vyfFm000: Title
| Field | Value |
|-------|-------|
| **Category** | code/testing/process/architecture |
| **Context** | describe when this applies |
| **Insight** | the fundamental discovery |
| **Suggested Action** | what to do next time |
| **Applies To** | file patterns or components |
| **Linked Knowledge IDs** | optional canonical IDs this insight builds on |
| **Observed At** | RFC3339 timestamp (e.g. 2026-02-22T12:00:00Z) |
| **Score** | 0.0-1.0 (impact significance) |
| **Confidence** | 0.0-1.0 (insight quality) |
| **Applied** | |
-->

## Observations

- The examples themselves were not syntactically wrong; the breakage came from implicit assumptions about running from the repository root and having `firecracker` plus the artifact-build tools on `PATH`.
- `port doctor` was already the correct preflight gate, so the most effective fix was to move that gate into the help text rather than inventing a new command or compatibility shim.
- Re-running the full help-published workflow in `nix develop` was important because it proved the examples were honest after the wording change instead of only looking better in static text.

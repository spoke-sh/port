---
created_at: 2026-03-06T14:44:58
---

# Reflection - Bootstrap Port Workspace And CLI

## Knowledge

### 1vye3K000: Verify Annotations Are Required For Story Evidence
| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Recording proof with `keel story record` against story acceptance criteria |
| **Insight** | `keel story record` ignores ACs unless each criterion includes an inline HTML comment that repeats the AC ID and declares the verification technique or command, for example `<!-- [SRS-01/AC-02] verify: cargo test -->`. |
| **Suggested Action** | Add verify annotations while authoring or refining stories, before starting implementation, so evidence recording and submit gates do not stall later. |
| **Applies To** | `.keel/stories/*/README.md` |
| **Linked Knowledge IDs** | |
| **Observed At** | 2026-03-06T22:47:00Z |
| **Score** | 0.83 |
| **Confidence** | 0.92 |
| **Applied** | yes |

## Observations

- The workspace split into CLI, model, and protocol crates was enough to make
  the command surface real without prematurely locking in runtime internals.
- `cargo test` only covered the default member while `default-members` was set;
  removing that shortcut made the repo-level test command match the board
  contract.
- `keel` transition and evidence commands are stateful and strict, so sequential
  execution is safer than parallel transitions even when the repo work itself is
  parallelizable.

---
created_at: 2026-03-07T18:10:45
---

# Reflection - Define Artifact Mobility Commands And Contracts

## Knowledge

### 1vz3rU000: Noninteractive Story Record Needs Editor Override
| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Recording `keel story record --cmd ...` proofs from the harness without an attached editor session |
| **Insight** | `keel story record` still opens a manual-evidence editor even for command proofs unless the editor exits immediately; setting `EDITOR=true` keeps the command proof path noninteractive. |
| **Suggested Action** | Use `EDITOR=true nix develop -c keel story record ... --cmd "<command>"` for automated proof capture and only fall back to a PTY editor when a manual note is genuinely needed. |
| **Applies To** | `.keel/stories/*`, proof recording workflow |
| **Linked Knowledge IDs** | |
| **Observed At** | 2026-03-08T02:10:00Z |
| **Score** | 0.80 |
| **Confidence** | 0.92 |
| **Applied** | yes |

## Observations

Modeling artifacts as logical references plus concrete variants was the right
cut. It let the CLI, docs, and runtime all use the same vocabulary for local
outputs, store paths, cache paths, and future remote backends instead of
grafting `push` and `pull` onto path-only specs.

The sample file-backed backend was valuable because it turned artifact mobility
into a real, testable operator flow rather than a placeholder command tree.
The push/pull round-trip through the actual `port` binary caught documentation
gaps immediately and gave the story a CLI-level proof that matches the user
experience.

The annoying part was `keel story record`, which is still editor-biased even
for command proofs. That is manageable once the editor override is known, but
it is easy to mistake for a hung process if you do not expect it.

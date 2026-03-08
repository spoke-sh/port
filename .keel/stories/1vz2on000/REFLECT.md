---
created_at: 2026-03-07T18:10:45
---

# Reflection - Define Artifact Mobility Commands And Contracts

## Knowledge

- [1vz3rU000](../../knowledge/1vz3rU000.md) Noninteractive Story Record Needs Editor Override

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

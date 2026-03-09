---
created_at: 2026-03-09T11:32:34
---

# Reflection - Publish OCI Artifact Operator Workflow

## Knowledge

### 1vzfPb000: Keel Story Evidence Commands Need An Explicit Repo Root
| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Recording story proof with `keel story record --cmd` for repo-scoped commands |
| **Insight** | `keel story record` does not guarantee execution from the repository root, so relative-path proof commands can fail even when the same command succeeds interactively from the shell |
| **Suggested Action** | Wrap repo-scoped proof commands in `bash -lc "cd /repo/root && ..."` or use absolute paths |
| **Applies To** | `.keel/stories/*`, `keel story record`, repo-local verification commands |
| **Linked Knowledge IDs** | |
| **Observed At** | 2026-03-09T19:32:34Z |
| **Score** | 0.67 |
| **Confidence** | 0.93 |
| **Applied** | AC-01 and AC-02 evidence for this story use an explicit `cd /home/alex/workspace/spoke-sh/port && ...` wrapper |

## Observations

The operator-facing workflow is now coherent because the same repo-local proof is visible in every product surface that matters: `port --help`, `README.md`, `docs/artifacts.md`, `examples/port.toml`, and the `just` helpers. That kept discovery and execution aligned instead of leaving OCI support hidden behind internal knowledge.

The most useful verification result was the real `just demo-push-oci && just demo-pull-oci` run. A direct cargo invocation had already worked, but the helper proof exposed the actual operator path and confirmed the repo-local registry flow was executable without falling back to a public registry.

The main surprise was process-related rather than runtime-related: `keel story record` executed the proof command outside the repo root, so relative-path commands failed until they were wrapped with an explicit `cd`. Recording evidence with explicit paths is the safer default for future stories.

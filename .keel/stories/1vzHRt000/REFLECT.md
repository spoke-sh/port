---
created_at: 2026-03-08T10:07:21
---

# Reflection - Model Pvm Node Capability Contract

## Knowledge

### 1w04a0000: Prefer Per-Ac Verify Annotations Over Shared Proof Blocks
| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | A story has multiple acceptance criteria and `keel story record` is used to capture rerunnable command proofs. |
| **Insight** | Shared summary-level verify comments can cause proof metadata to drift or cross-wire between acceptance criteria, while one inline verify annotation per AC stays stable. |
| **Suggested Action** | For multi-AC stories, use repo-rooted `verify-ac-*.sh` scripts and only the per-AC inline verify comment form before recording evidence. |
| **Applies To** | `.keel/stories/*/README.md`, `.keel/stories/*/verify-ac-*.sh` |
| **Linked Knowledge IDs** | 1w03x0000 |
| **Observed At** | 2026-03-08T17:07:30Z |
| **Score** | 0.72 |
| **Confidence** | 0.9 |
| **Applied** | yes |

## Observations

The cleanest implementation path was to keep the rich local
`FirecrackerPvmLaneContract` intact, introduce one smaller hosted-node
capability contract plus a shared capability-state enum, and then reuse that
state vocabulary in both places.

The proof-first loop worked as intended here. The failing tests forced the
exact surface area: model state, sample config, hosted inventory serialization,
and nothing broader. The only friction was `keel story record` metadata drift
when a shared verify block was present, which is why the per-AC annotation form
is now the preferred pattern.

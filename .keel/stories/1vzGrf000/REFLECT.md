---
created_at: 2026-03-08T09:43:44
---

# Reflection - Materialize PVM Artifact Variants

## Knowledge

### 1w03w0000: Infer Artifact Variants From Canonical Output Paths
| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Artifact scripts are invoked with a single output path, but the build and validation logic still needs to know architecture and protection mode without growing a second command contract. |
| **Insight** | Deriving selector intent from the canonical artifact path keeps model selection, cache/store layout, and script behavior aligned. That lets new variants like `x86_64/firecracker/pvm` land without widening the script API. |
| **Suggested Action** | When adding future artifact selectors, keep the path layout canonical and let the scripts derive selector intent from it before introducing new script arguments or hidden environment variables. |
| **Applies To** | `scripts/artifacts/*.sh`, `crates/port-runtime/src/lib.rs`, artifact selector evolution |
| **Linked Knowledge IDs** | 1w03v0000 |
| **Observed At** | 2026-03-08T09:45:00Z |
| **Score** | 0.8 |
| **Confidence** | 0.92 |
| **Applied** | yes |

## Observations

Keeping the scripts keyed off the output path worked well. The model and
runtime already treat the selector path as canonical, so the PVM variant work
could stay focused on one contract instead of adding more script flags.

The most important follow-on was the runtime launch boundary. Once the PVM
artifacts existed, launch could no longer honestly fail with “missing
artifact”. Moving that failure to the unprepared PVM host-kit preflight kept
the product surface truthful while still shipping the artifact foundation.

---
created_at: 2026-03-08T09:35:39
---

# Reflection - Add PVM Doctor Host Kit Checks

## Knowledge

### 1w03vv000: Add Probe Seams Before Expanding Host Diagnostics
| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | Host diagnostics depend on live OS facts like platform, architecture, `/proc/cmdline`, or binary lookup, but the story needs deterministic tests for several incompatible states. |
| **Insight** | A small probe struct is enough to turn environment-dependent diagnostics into a testable seam. That is lower-cost and more maintainable than trying to mock shell commands or `/proc` access ad hoc in each test. |
| **Suggested Action** | When extending `doctor` with more host or platform checks, first introduce or reuse a single fact-gathering struct and keep the decision logic pure over that struct. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, future doctor and platform readiness checks |
| **Linked Knowledge IDs** | 1w03v0000 |
| **Observed At** | 2026-03-08T09:36:00Z |
| **Score** | 0.82 |
| **Confidence** | 0.94 |
| **Applied** | yes |

## Observations

The main improvement was separating the environment probe from the doctor logic.
Once the host facts were explicit, it became straightforward to prove both the
happy path and the fail-fast path for `pti=off`, architecture mismatch, and the
patched Firecracker binary contract.

The other useful correction was switching the story verification commands to
repo-rooted scripts. `keel story record` did not run from the repo root, so a
relative `examples/port.toml` path was not reliable enough for a submit gate.

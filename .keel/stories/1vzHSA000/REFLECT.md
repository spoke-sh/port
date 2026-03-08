---
created_at: 2026-03-08T10:14:10
---

# Reflection - Select Local Pvm Runtime Inputs

## Knowledge

### 1w04f0000: Split Lane-Specific Binary Selection Into A Pure Helper
| Field | Value |
|-------|-------|
| **Category** | code |
| **Context** | A launch path needs to choose different VMM binaries for different protection modes, but spawning the VMM is too expensive and environment-sensitive for most unit tests. |
| **Insight** | Pulling lane-specific binary selection into a pure helper makes the protection-mode contract testable without depending on a live Firecracker process or host PATH mutations. |
| **Suggested Action** | When adding more substrate or protection-mode launch inputs, isolate the selection logic in a pure helper before wiring it into the process-spawn path. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, launch-path selection helpers |
| **Linked Knowledge IDs** | 1w04a0000 |
| **Observed At** | 2026-03-08T17:15:30Z |
| **Score** | 0.75 |
| **Confidence** | 0.93 |
| **Applied** | yes |

## Observations

The smallest useful change was not in the doctor path; it was in the launch
path. Port already knew how to diagnose missing PVM prerequisites, but it still
selected the standard Firecracker binary for all local launches. Pulling binary
selection into its own helper closed that gap without widening the story into a
full launch refactor.

The CLI proof was worth keeping as a real command flow instead of a pure unit
test. It confirmed the operator-visible behavior now says exactly what we need:
the host kit is unprepared, `pti=off` is missing, and `firecracker-pvm` is not
a compatible fallback.

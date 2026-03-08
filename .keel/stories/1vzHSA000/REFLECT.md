---
created_at: 2026-03-08T10:14:10
---

# Reflection - Select Local Pvm Runtime Inputs

## Knowledge

- [1w04f0000](../../knowledge/1w04f0000.md) Split Lane-Specific Binary Selection Into A Pure Helper

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

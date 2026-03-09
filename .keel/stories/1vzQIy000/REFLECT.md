---
created_at: 2026-03-08T20:57:59
---

# Reflection - Publish Hosted Detached Forward Workflow

## Knowledge

- [1w04h0000](../../knowledge/1w04h0000.md) Keep Hosted Demo Socket Paths Short Under Nested Nix Shells

## Observations

The product change itself was small, but the verification path exposed two real
workflow defects: the demo script used `cargo run` in several backgrounded
steps with short readiness waits, and its default temp root inherited a very
long nested-Nix `TMPDIR`. That combination made `keel verify` and `keel story
record` fail even though the direct proof passed.

Prebuilding `port` and `port-guest-agent`, then switching the demo to a short
`/tmp/porthd.*` workdir, made the hosted detached-forward proof deterministic
across direct runs, `keel verify`, and story evidence capture. The remaining
process annoyance is still the `keel story record` README mutation bug, so the
story bundle needed one manual fix before submit.

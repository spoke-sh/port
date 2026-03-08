---
created_at: 2026-03-08T09:43:44
---

# Reflection - Materialize PVM Artifact Variants

## Knowledge

- [1w03w0000](../../knowledge/1w03w0000.md) Infer Artifact Variants From Canonical Output Paths

## Observations

Keeping the scripts keyed off the output path worked well. The model and
runtime already treat the selector path as canonical, so the PVM variant work
could stay focused on one contract instead of adding more script flags.

The most important follow-on was the runtime launch boundary. Once the PVM
artifacts existed, launch could no longer honestly fail with “missing
artifact”. Moving that failure to the unprepared PVM host-kit preflight kept
the product surface truthful while still shipping the artifact foundation.

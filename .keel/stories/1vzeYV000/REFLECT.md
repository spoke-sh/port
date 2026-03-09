---
created_at: 2026-03-09T10:53:25
---

# Reflection - Define OCI Registry Artifact Contract

## Knowledge

- [1vzeZ1000](../../knowledge/1vzeZ1000.md) Derive OCI Variant References In The Model

## Observations

- The cleanest contract split was to make `oci-registry` resolvable now, but keep transport execution for the next stories. That let the model, doctor, and runtime resolver become real without smuggling half-finished push/pull behavior into this slice.
- The useful failure boundary is before any command spawn: model validation rejects empty auth variable names, doctor reports missing `oras` and missing auth env explicitly, and runtime resolution fails with the resolved remote reference and auth mode instead of the old reserved-backend stub.
- The generated `.port/` directories under crate paths came from tests rather than product state, so they had to be removed before story submission to keep the worktree clean and the commit scoped.

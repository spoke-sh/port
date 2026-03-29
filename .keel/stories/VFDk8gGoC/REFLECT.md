---
created_at: 2026-03-28T21:32:05
---

# Reflection - Stage Offline K3s Artifacts And Guest Profile

## Knowledge

- [VFDUWw5P4](../../knowledge/VFDUWw5P4.md) Local guest-agent execs need guest-root-relative paths

## Observations

The cleanest way to keep the first local cluster slice honest was to make the
offline bootstrap kit explicit in the cluster contract instead of hiding it in
ad hoc runtime defaults. That let the CLI surface, runtime stage helper, and
sample config all point at the same Port-owned inputs.

Two integration edges mattered more than the code volume. First, the checked-in
bootstrap kit paths needed deterministic repo-root resolution or the runtime
tests would fail when executed from a crate directory. Second, the fake local
guest-agent copy path behaves like a guest filesystem, but execs only become
truthful when commands run from guest `/` with root-relative paths.

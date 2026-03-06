---
created_at: 2026-03-06T15:52:06
---

# Reflection - Model Cloud Linux Providers

## Knowledge

- [1vyezc000](../../knowledge/1vyezc000.md) Parse-Test Canonical Example Configs

## Observations

- Adding provider identity at `HostSpec` was the right seam because later CLI/runtime diagnostics can now branch on declared intent instead of inferring cloud meaning from `connection.mode = ssh`.
- Extending the canonical example config with remote provider hosts and machine stubs gives later cloud stories stable names to verify against without disturbing the existing local `demo` workflow.
- Using an isolated `CARGO_TARGET_DIR` kept verification from mutating the accidentally tracked repo-local `target/` tree while the MVP work continues.

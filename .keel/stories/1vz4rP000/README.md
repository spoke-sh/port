---
id: 1vz4rP000
title: Gate Linux Only Dev Shell Inputs
type: feat
status: in-progress
created_at: 2026-03-07T19:31:19
updated_at: 2026-03-07T19:31:53
scope: 1vz3ck000/1vz4qH000
started_at: 2026-03-07T19:31:53
---

# Gate Linux Only Dev Shell Inputs

## Summary

Keep `nix develop` usable on macOS by gating Linux-only runtime packages in the
default dev shell while preserving the Linux toolchain Port still needs for
local Firecracker launch.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] `flake.nix` no longer attempts to evaluate unsupported Linux-only runtime packages on macOS, and Darwin shell evaluation succeeds without unsupported-system overrides.
- [ ] [SRS-02/AC-01] The Linux shell still includes Firecracker and Linux networking/runtime tools required by Port's local launch workflow.
- [ ] [SRS-03/AC-01] Docs or shell messaging explain that the macOS shell is for repo tooling while Linux-only runtime tools remain omitted.

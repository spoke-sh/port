---
id: 1vyfvx000
title: Rework Copy And Forward For Live Guest Transport
type: feat
status: done
created_at: 2026-03-06T16:54:21
updated_at: 2026-03-06T17:35:18
scope: 1vydg7000/1vyfve000
started_at: 2026-03-06T17:12:09
submitted_at: 2026-03-06T17:35:11
completed_at: 2026-03-06T17:35:18
---

# Rework Copy And Forward For Live Guest Transport

## Summary

Replace the shared-host-path and guest-local-listener assumptions in `copy` and
`forward` with behaviors that stay coherent across a real host/guest boundary.

## Acceptance Criteria

<!-- verify: manual, SRS-04:start:end, proof: ac-1.log-->
- [x] [SRS-04/AC-01] `port guest copy --direction host-to-guest` transfers file contents into the launched VM without requiring the guest to see the host source path. <!-- [SRS-04/AC-01] verify: nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime copy_guest_file_uses_firecracker_vsock_tunnel_in_both_directions, proof: ac-1.log-->
<!-- verify: manual, SRS-04:start:end, proof: ac-2.log-->
- [x] [SRS-04/AC-02] `port guest copy --direction guest-to-host` transfers file contents back to the host through the canonical CLI and model. <!-- [SRS-04/AC-02] verify: nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-cli cli_guest_commands_cover_all_capabilities, proof: ac-2.log-->
<!-- verify: manual, SRS-05:start:end, proof: ac-3.log-->
- [x] [SRS-05/AC-01] `port guest forward` binds and serves on the host side, proxies to a guest target through the live transport, and behaves according to documented lifecycle expectations. <!-- [SRS-05/AC-01] verify: nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime guest_forward_session_proxies_through_firecracker_vsock_tunnel, proof: ac-3.log-->
<!-- verify: manual, SRS-06:start:end, proof: ac-4.log-->
- [x] [SRS-06/AC-01] `port --help`, README, and operator docs describe the live guest transport and the current `guest forward` lifecycle accurately. <!-- [SRS-06/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-cli help_includes_primary_surfaces && rg -n "foreground host-side proxy|ifconfig lo up" README.md docs/operators.md crates/port-cli/src/lib.rs', proof: ac-4.log-->

---
id: 1vyfwJ000
title: Stabilize Runtime State For Guest Transport
type: fix
status: done
created_at: 2026-03-06T16:54:43
updated_at: 2026-03-06T17:02:22
scope: 1vydg7000/1vyfve000
started_at: 2026-03-06T16:58:03
submitted_at: 2026-03-06T17:02:19
completed_at: 2026-03-06T17:02:22
---

# Stabilize Runtime State For Guest Transport

## Summary

Harden the local runtime surface that the live guest transport depends on:
clean stale runtime state before relaunch and replace the generic missing-socket
guest error with an actionable launched-VM transport diagnostic.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-01/AC-01] `port machine launch` removes stale runtime pid/vsock/socket files left by dead Firecracker runs before attempting a relaunch. <!-- [SRS-01/AC-01] verify: nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime prepare_runtime_state_cleans_stale_socket_and_pid_files, proof: ac-1.log-->
<!-- verify: manual, SRS-01:start:end, proof: ac-2.log-->
- [x] [SRS-01/AC-02] `port machine launch` fails with an explicit "already running" message when the requested machine still has a live Firecracker process under the same runtime root. <!-- [SRS-01/AC-02] verify: nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime prepare_runtime_state_rejects_live_matching_firecracker_process, proof: ac-2.log-->
<!-- verify: manual, SRS-01:start:end, proof: ac-3.log-->
- [x] [SRS-01/AC-03] `port guest ...` against a launched VM without a connected live transport returns an actionable error that points at the launched-VM transport gap instead of only reporting a missing host socket. <!-- [SRS-01/AC-03] verify: nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime guest_operations_explain_missing_live_vm_transport, proof: ac-3.log-->

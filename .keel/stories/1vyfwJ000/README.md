---
id: 1vyfwJ000
title: Stabilize Runtime State For Guest Transport
type: fix
status: backlog
created_at: 2026-03-06T16:54:43
updated_at: 2026-03-06T16:56:57
scope: 1vydg7000/1vyfve000
---

# Stabilize Runtime State For Guest Transport

## Summary

Harden the local runtime surface that the live guest transport depends on:
clean stale runtime state before relaunch and replace the generic missing-socket
guest error with an actionable launched-VM transport diagnostic.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] `port machine launch` removes stale runtime pid/vsock/socket
      files left by dead Firecracker runs before attempting a relaunch.
- [ ] [SRS-01/AC-02] `port machine launch` fails with an explicit “already
      running” message when the requested machine still has a live Firecracker
      process under the same runtime root.
- [ ] [SRS-01/AC-03] `port guest ...` against a launched VM without a connected
      live transport returns an actionable error that points at the launched-VM
      transport gap instead of only reporting a missing host socket.

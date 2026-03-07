---
id: 1vyfvx000
title: Rework Copy And Forward For Live Guest Transport
type: feat
status: backlog
created_at: 2026-03-06T16:54:21
updated_at: 2026-03-06T16:56:57
scope: 1vydg7000/1vyfve000
---

# Rework Copy And Forward For Live Guest Transport

## Summary

Replace the shared-host-path and guest-local-listener assumptions in `copy` and
`forward` with behaviors that stay coherent across a real host/guest boundary.

## Acceptance Criteria

- [ ] [SRS-04/AC-01] `port guest copy --direction host-to-guest` transfers file
      contents into the launched VM without requiring the guest to see the host
      source path.
- [ ] [SRS-04/AC-02] `port guest copy --direction guest-to-host` transfers file
      contents back to the host through the canonical CLI and model.
- [ ] [SRS-05/AC-01] `port guest forward` binds and serves on the host side,
      proxies to a guest target through the live transport, and behaves
      according to documented lifecycle expectations.
- [ ] [SRS-06/AC-01] `port --help`, README, and operator docs describe the live
      guest transport and the current `guest forward` lifecycle accurately.

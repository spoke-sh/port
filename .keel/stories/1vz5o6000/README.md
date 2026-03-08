---
id: 1vz5o6000
title: Add Detached And Unix-Socket Forwarding
type: feat
status: backlog
created_at: 2026-03-07T20:31:58
updated_at: 2026-03-07T20:35:30
scope: 1vz4Yn000/1vz5mg000
---

# Add Detached And Unix-Socket Forwarding

## Summary

Extend the canonical forwarding surface with detached and Unix-socket modes once
the hosted guest runtime path exists.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] `guest forward` supports detached lifecycle management and Unix-socket forwarding without introducing a second forwarding command family.
- [ ] [SRS-03/AC-02] CLI help, docs, and evidence explain how detached and Unix-socket forwarding relate to the hosted guest runtime path and what remains downstream.

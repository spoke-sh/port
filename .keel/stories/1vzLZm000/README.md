---
id: 1vzLZm000
title: Wire Avf Guest Transport And Console Capture
type: feat
status: backlog
created_at: 2026-03-08T14:22:14
updated_at: 2026-03-08T14:23:39
scope: 1vzJKE000/1vzLYD000
---

# Wire Avf Guest Transport And Console Capture

## Summary

Map the shared guest protocol onto the AVF transport and serial-console
surfaces so the canonical `guest` verbs and machine log inspection work for
AVF-backed machines.

## Acceptance Criteria

- [ ] [SRS-04/AC-01] AVF-targeted machines expose `guest exec|copy|pty|logs|forward`
  through the canonical CLI and shared guest protocol via an AVF transport
  adapter.
- [ ] [SRS-04/AC-02] AVF boot and console output land in canonical runtime log
  surfaces that `machine status` and operator inspection can reference.

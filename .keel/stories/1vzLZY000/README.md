---
id: 1vzLZY000
title: Publish Macos Avf Operator Workflow
type: feat
status: backlog
created_at: 2026-03-08T14:22:00
updated_at: 2026-03-08T14:23:39
scope: 1vzJKE000/1vzLYD000
---

# Publish Macos Avf Operator Workflow

## Summary

Publish the native macOS AVF workflow across the CLI help and docs once the
runtime slices are in place, including proof commands and explicit unsupported
boundaries.

## Acceptance Criteria

- [ ] [SRS-05/AC-01] CLI help, README, `docs/avf.md`, and macOS operator docs
  describe the native AVF workflow, prerequisites, and unsupported boundaries
  through the canonical `port` command model.
- [ ] [SRS-05/AC-02] Recorded proof demonstrates the AVF workflow contract
  while also preserving explicit Linux-lane and unsupported-host boundaries for
  operators.

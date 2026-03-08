---
id: 1vzLZj000
title: Implement Avf Local Machine Driver
type: feat
status: backlog
created_at: 2026-03-08T14:22:11
updated_at: 2026-03-08T14:23:39
scope: 1vzJKE000/1vzLYD000
---

# Implement Avf Local Machine Driver

## Summary

Add the first AVF local machine driver behind the shared runtime seam so
`machine launch`, `status`, and `stop` can own AVF-backed VMs without
introducing a substrate-specific command family.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] `port machine launch`, `status`, and `stop` route
  AVF-targeted machines through a local AVF driver that writes canonical
  runtime manifests plus AVF-specific runtime metadata.
- [ ] [SRS-03/AC-02] The AVF local driver keeps deterministic runtime metadata
  and explicit substrate-specific failure detail instead of falling back
  silently to existing Linux lanes.

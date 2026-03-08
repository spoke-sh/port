---
id: 1vzMXo000
title: Implement Streamed Pty And Log Follow
type: feat
status: backlog
created_at: 2026-03-08T15:24:16
updated_at: 2026-03-08T15:25:58
scope: 1vzMVF000/1vzMVY000
---

# Implement Streamed Pty And Log Follow

## Summary

Implement the stream-capable PTY and log-follow guest paths through the shared
guest protocol, guest agent, runtime, and canonical CLI.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] `port guest pty` provides streamed interactive behavior
  through the canonical CLI and shared guest protocol for local and AVF-backed
  runtimes.
- [ ] [SRS-02/AC-02] `port guest logs --follow` streams incremental guest log
  output while preserving the existing non-follow log behavior.

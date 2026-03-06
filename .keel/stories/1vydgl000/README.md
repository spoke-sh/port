---
id: 1vydgl000
title: Bootstrap Port Workspace And CLI
type: feat
status: icebox
created_at: 2026-03-06T14:30:31
updated_at: 2026-03-06T14:32:49
scope: 1vydg7000/1vydgL000
---

# Bootstrap Port Workspace And CLI

## Summary

Create the Rust workspace, canonical `port` CLI skeleton, and shared model that
subsequent runtime, guest-agent, and artifact stories can extend without
rewriting the command surface.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] A Rust workspace exists with the canonical `port` binary and shared model/protocol crates checked in.
- [ ] [SRS-01/AC-02] `port --help` and the top-level command tree expose the planned artifact, machine, and guest surfaces with coherent help text.
- [ ] [SRS-01/AC-03] Model serialization and CLI parsing are covered by automated tests runnable through the repo test command.

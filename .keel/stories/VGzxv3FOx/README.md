---
# system-managed
id: VGzxv3FOx
status: done
created_at: 2026-04-16T16:11:05
updated_at: 2026-04-16T16:54:52
# authored
title: Add Ping Frame And Guest-Agent Heartbeat Wire Contract
type: feat
operator-signal:
scope: VGzxKV9OX/VGzxkoGrw
index: 1
started_at: 2026-04-16T16:50:03
submitted_at: 2026-04-16T16:54:49
completed_at: 2026-04-16T16:54:52
---

# Add Ping Frame And Guest-Agent Heartbeat Wire Contract

## Summary

Introduce the minimal wire-level contract that lets the node-agent prove a guest-agent is awake without piggybacking on `Exec` or any streamed operation. Add a `Ping` request frame and a `Pong` response (or equivalent envelope pair) to `port-agent-protocol`, and wire a handler in `port-guest-agent` that responds immediately on its existing read loop. This story owns only the protocol and the guest-side handler; the node-agent-side probe loop lives in a follow-on story.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end -->
- [x] [SRS-01/AC-01] `port-agent-protocol` defines the `Ping`/`Pong` frame pair and round-trips through serde; `port-guest-agent`'s read loop matches `Ping` and writes `Pong` without touching running managed services or PTY sessions, with a documented, observable response budget. <!-- [SRS-01/AC-01] verify: cargo test -p port-agent-protocol -p port-guest-agent, proof: ac-1.log -->

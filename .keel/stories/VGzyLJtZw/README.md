---
# system-managed
id: VGzyLJtZw
status: backlog
created_at: 2026-04-16T16:12:46
updated_at: 2026-04-16T16:15:21
# authored
title: Drive Periodic Guest Heartbeat Probe From Node-Agent
type: feat
operator-signal:
scope: VGzxKV9OX/VGzxkoGrw
index: 2
---

# Drive Periodic Guest Heartbeat Probe From Node-Agent

## Summary

Once the wire contract exists, the node-agent has to actually issue pings on a timer and record the result. Spawn a per-machine probe task on registration, issue `Ping` on the existing transport with a bounded interval, stamp `guest_agent_last_heartbeat` in a per-machine in-memory sidecar on successful `Pong`, and leave the prior timestamp untouched on any failure (timeout, decode error, transport close). The probe must run independently of in-flight guest operations so a long `Exec` or `Pty` does not mask a wedged guest-agent.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] The node-agent spawns a per-machine probe task on registration and cancels it on deregistration; successful pongs update a `RwLock<BTreeMap<String, Instant>>` sidecar keyed by machine name, and failed probes leave the prior timestamp untouched. <!-- [SRS-02/AC-01] verify: cargo test -p port-runtime -- guest_heartbeat_probe, proof: ac-1.log -->
- [ ] [SRS-NFR-01/AC-01] The probe loop runs concurrently with long-running guest operations: a test keeps `Exec` or `Pty` open for longer than one probe interval and asserts the guest heartbeat still advances, proving the probe does not serialize behind in-flight ops. <!-- [SRS-NFR-01/AC-01] verify: cargo test -p port-runtime -- guest_heartbeat_concurrent_with_exec, proof: ac-2.log -->

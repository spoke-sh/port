---
# system-managed
id: VGzyLTlgJ
status: backlog
created_at: 2026-04-16T16:12:47
updated_at: 2026-04-16T16:15:21
# authored
title: Surface Guest Refresh Age Seconds In Cluster Status
type: feat
operator-signal:
scope: VGzxKV9OX/VGzxkoGrw
index: 3
---

# Surface Guest Refresh Age Seconds In Cluster Status

## Summary

With the probe loop stamping heartbeats, extend the hosted cluster status contract to surface `guest_refresh_age_seconds: Option<u64>` per machine. Compute the age from the monotonic `Instant` sidecar so wall-clock jumps on the node-agent host cannot produce negative or inflated values. Thread the field through the existing node-agent → control-plane status path the same way `refresh_age_seconds` already flows, and confirm the existing guest-operation suites (`exec`, `copy`, `pty`, `logs`, `forward`, hosted round-trips) still pass with the probe loop running.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] `port cluster status --format json` and the machine status contract expose `guest_refresh_age_seconds: Option<u64>` per machine, `None` before the first successful pong and populated thereafter; a new integration test asserts the field appears after a probe and increases monotonically across reads. <!-- [SRS-03/AC-01] verify: cargo test -p port-runtime -- guest_refresh_age_seconds_surfaces, proof: ac-1.log -->
- [ ] [SRS-04/AC-01] The existing hosted guest-operation test suites (including the `exec`, `copy`, `pty`, `logs`, `forward`, and hosted control-plane round-trip tests) continue to pass with the probe loop active. <!-- [SRS-04/AC-01] verify: cargo test -p port-runtime --lib, proof: ac-2.log -->
- [ ] [SRS-NFR-02/AC-01] The age computation uses a monotonic clock source (e.g. `Instant::now()` saturating-subtracted from the sidecar entry); a unit test covers the behavior under a simulated wall-clock jump and confirms no regression to wall-clock arithmetic. <!-- [SRS-NFR-02/AC-01] verify: cargo test -p port-runtime -- guest_refresh_age_seconds_uses_monotonic_clock, proof: ac-3.log -->

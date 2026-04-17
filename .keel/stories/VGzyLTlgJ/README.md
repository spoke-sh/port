---
# system-managed
id: VGzyLTlgJ
status: done
created_at: 2026-04-16T16:12:47
updated_at: 2026-04-16T17:07:56
# authored
title: Surface Guest Refresh Age Seconds In Cluster Status
type: feat
operator-signal:
scope: VGzxKV9OX/VGzxkoGrw
index: 3
started_at: 2026-04-16T17:01:39
submitted_at: 2026-04-16T17:07:56
completed_at: 2026-04-16T17:07:56
---

# Surface Guest Refresh Age Seconds In Cluster Status

## Summary

With the probe loop stamping heartbeats, extend the hosted cluster status contract to surface `guest_refresh_age_seconds: Option<u64>` per machine. Compute the age from the monotonic `Instant` sidecar so wall-clock jumps on the node-agent host cannot produce negative or inflated values. Thread the field through the existing node-agent → control-plane status path the same way `refresh_age_seconds` already flows, and confirm the existing guest-operation suites (`exec`, `copy`, `pty`, `logs`, `forward`, hosted round-trips) still pass with the probe loop running.

## Acceptance Criteria

<!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
- [x] [SRS-03/AC-01] `port cluster status --format json` and the machine status contract expose `guest_refresh_age_seconds: Option<u64>` per machine, `None` before the first successful pong and populated thereafter; an integration test asserts the field appears after a probe and increases monotonically across reads. <!-- [SRS-03/AC-01] verify: cargo test -p port-runtime -- guest_heartbeat_age, proof: ac-2.log -->
<!-- verify: manual, SRS-04:start:end, proof: ac-3.log-->
- [x] [SRS-04/AC-01] The existing hosted guest-operation test suites (including the `exec`, `copy`, `pty`, `logs`, `forward`, and hosted control-plane round-trip tests) continue to pass with the probe loop active. <!-- [SRS-04/AC-01] verify: cargo test -p port-runtime --lib, proof: ac-2.log -->
<!-- verify: manual, SRS-NFR-02:start:end -->
- [x] [SRS-NFR-02/AC-01] The age computation uses a monotonic clock source (`Instant::now()` saturating-subtracted from the sidecar entry); `guest_heartbeat_age_uses_monotonic_clock_and_is_non_regressive` asserts non-regression across reads. <!-- [SRS-NFR-02/AC-01] verify: cargo test -p port-runtime -- guest_heartbeat_age_uses_monotonic_clock, proof: ac-3.log -->

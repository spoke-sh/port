---
# system-managed
id: VGafyVDGA
status: backlog
created_at: 2026-04-12T08:23:00
updated_at: 2026-04-12T08:28:05
# authored
title: Capture Hosted AWS PVM Failover Proof For The Stable Endpoint
type: feat
operator-signal:
scope: VGYFpfmpi/VGafx2vn4
index: 2
---

# Capture Hosted AWS PVM Failover Proof For The Stable Endpoint

## Summary

Capture one human-reviewable failover proof for the hosted AWS PVM HA endpoint
so Port's first real-HA claim is backed by executable evidence rather than a
documentation promise.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] The canonical proof workflow shows the stable endpoint working before and after one supported control-plane host-loss or guest-replacement scenario on hosted AWS PVM. <!-- verify: command, SRS-03:start:end -->
- [ ] [SRS-NFR-01/AC-02] The failover proof is stored as a human-reviewable Port proof artifact rather than as chat-only notes. <!-- verify: manual, SRS-NFR-01:start:end -->
- [ ] [SRS-NFR-02/AC-03] The proof or its paired negative-path evidence makes missing failover prerequisites explicit instead of implying stability that Port cannot yet provide. <!-- verify: command, SRS-NFR-02:start:end -->

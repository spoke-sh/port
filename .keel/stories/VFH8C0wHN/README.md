---
# system-managed
id: VFH8C0wHN
status: done
created_at: 2026-03-29T09:40:46
updated_at: 2026-03-29T10:08:29
# authored
title: Repair Local Cluster Guest Boot Path
type: feat
operator-signal:
scope: VFH7YspJx/VFH7t3cG9
index: 1
started_at: 2026-03-29T09:48:13
submitted_at: 2026-03-29T10:08:25
completed_at: 2026-03-29T10:08:29
---

<!-- verify: manual, SRS-01:start:end, proof: ac-1.cluster-up.json, ac-1.cluster-down.json -->

# Repair Local Cluster Guest Boot Path

## Summary

Repair the shipped local guest image or boot wiring so the checked-in
single-node cluster lane boots cleanly through `/init` on Linux instead of
panicking before cluster bootstrap can begin.

## Acceptance Criteria

- [x] [SRS-01/AC-01] `port --config examples/port.toml cluster up --cluster demo --runtime-root <tmp> --format json` succeeds on Linux without Firecracker exiting during boot or the guest failing `Run /init as init process`. <!-- [SRS-01/AC-01] verify: manual, proof: ac-1.cluster-up.json, ac-1.cluster-down.json -->

---
# system-managed
id: VFH8C0wHN
status: backlog
created_at: 2026-03-29T09:40:46
updated_at: 2026-03-29T09:41:57
# authored
title: Repair Local Cluster Guest Boot Path
type: feat
operator-signal:
scope: VFH7YspJx/VFH7t3cG9
index: 1
---

# Repair Local Cluster Guest Boot Path

## Summary

Repair the shipped local guest image or boot wiring so the checked-in
single-node cluster lane boots cleanly through `/init` on Linux instead of
panicking before cluster bootstrap can begin.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] `port --config examples/port.toml cluster up --cluster demo --runtime-root <tmp> --format json` succeeds on Linux without Firecracker exiting during boot or the guest failing `Run /init as init process`. <!-- verify: manual, SRS-01:start:end -->

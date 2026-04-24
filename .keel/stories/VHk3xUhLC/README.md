---
# system-managed
id: VHk3xUhLC
status: done
created_at: 2026-04-24T13:23:36
updated_at: 2026-04-24T13:28:37
# authored
title: Restart Managed Services After Crashed Child Stalls Healthcheck Cleanup
type: fix
operator-signal:
started_at: 2026-04-24T13:23:40
completed_at: 2026-04-24T13:28:37
---

# Restart Managed Services After Crashed Child Stalls Healthcheck Cleanup

## Summary

Ensure the guest-agent managed service supervisor restarts a crashed service
even when an in-flight healthcheck subprocess is still running or stuck.

## Acceptance Criteria

- [x] [SRS-NFR-01/AC-01] Managed services with `Always` restart policy are restarted when the child exits during an active healthcheck, without blocking on healthcheck subprocess cleanup. <!-- verify: command, SRS-NFR-01:start:end, proof: ac-1.log -->

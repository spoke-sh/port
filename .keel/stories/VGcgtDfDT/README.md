---
# system-managed
id: VGcgtDfDT
status: backlog
created_at: 2026-04-12T16:39:11
updated_at: 2026-04-12T16:39:43
# authored
title: Enforce Managed Service Ownership For Hosted K3s
type: feat
operator-signal:
scope: VGcgU9T57/VGcghwZrb
index: 2
---

# Enforce Managed Service Ownership For Hosted K3s

## Summary

Remove legacy detached hosted K3s paths from the valid runtime contract so
hosted workers and servers exist under managed Port service ownership only.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] Port rejects, replaces, or otherwise eliminates legacy detached hosted K3s paths in favor of managed-service ownership. <!-- verify: manual, SRS-02:start:end -->
- [ ] [SRS-NFR-01/AC-02] Managed-service ownership remains explicit in runtime artifacts and service status after the cutover. <!-- verify: manual, SRS-NFR-01:start:end -->

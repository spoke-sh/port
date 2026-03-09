---
id: 1vzTTK000
title: Surface Registered Placement Through Machine Commands
type: feat
status: done
created_at: 2026-03-08T22:48:06
updated_at: 2026-03-09T00:03:03
scope: 1vzTQB000/1vzTR9000
started_at: 2026-03-08T23:44:26
completed_at: 2026-03-09T00:03:03
---

# Surface Registered Placement Through Machine Commands

## Summary

Surface registered-node identity, placement detail, and stale-registration
failures through canonical hosted `port machine` output instead of a separate
fleet-only surface.

## Acceptance Criteria

<!-- verify: command, SRS-04:start, proof: ac-1.log -->
- [x] [SRS-04/AC-01] Hosted `port machine list|status|monitor|stop` surface selected registered-node identity, freshness or registration state, and placement detail through canonical machine output. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzTTK000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-04:end, proof: ac-2.log -->
- [x] [SRS-04/AC-02] Missing or stale registered-node state remains operator-visible through machine output instead of collapsing into generic hosted transport failures. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzTTK000/verify-ac-2.sh, proof: ac-2.log -->

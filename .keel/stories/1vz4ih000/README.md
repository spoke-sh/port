---
id: 1vz4ih000
title: Sequence Hosted Follow-On Work
type: feat
status: done
created_at: 2026-03-07T19:22:19
updated_at: 2026-03-07T20:37:02
scope: 1vz4Yn000/1vz4cU000
started_at: 2026-03-07T20:29:46
submitted_at: 2026-03-07T20:36:52
completed_at: 2026-03-07T20:37:02
---

# Sequence Hosted Follow-On Work

## Summary

Sequence the follow-on hosted-control backlog so monitoring, secrets, services,
sandboxes, detached forwarding, Unix-socket forwarding, and SDK work remain
ordered behind the first hosted auth, inventory, lifecycle, and guest-bridge
foundation.

## Acceptance Criteria

<!-- verify: manual, SRS-05:start:end, proof: ac-1.log, ac-2.log -->
- [x] [SRS-05/AC-01] The hosted-control board records an implementation-ready follow-on sequence for monitoring, secrets, services, sandboxes, detached forwarding, Unix-socket forwarding, and SDK work after this voyage. <!-- [SRS-05/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz4ih000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-05/AC-02] README or hosted-control docs explain that those follow-on capabilities are downstream of the authenticated API, inventory, lifecycle, and guest-bridge foundation rather than already shipped. <!-- [SRS-05/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz4ih000/verify-ac-2.sh, proof: ac-2.log -->

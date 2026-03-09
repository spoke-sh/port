---
id: 1vzQJB000
title: Route Hosted Detached Forward Lifecycle
type: feat
status: backlog
created_at: 2026-03-08T19:25:25
updated_at: 2026-03-08T19:27:01
scope: 1vzETR000/1vzQEj000
---

# Route Hosted Detached Forward Lifecycle

## Summary

Route hosted detached guest forward lifecycle actions through the canonical CLI
and SDK so hosted start, list, stop, and `--name` all use the live control
plane and node-agent path.

## Acceptance Criteria

<!-- verify: command, SRS-03:start:end, proof: ac-1.log -->
- [ ] [SRS-03/AC-01] Hosted `port guest forward --lifecycle detached [--name ...]` uses the live control-plane and node-agent path while preserving the existing command family. <!-- [SRS-03/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzQJB000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-03:start:end, proof: ac-2.log -->
- [ ] [SRS-03/AC-02] Hosted `port guest forward --list` and `--stop --name ...` use the live hosted transport and no longer fall back to repo-local lifecycle state. <!-- [SRS-03/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzQJB000/verify-ac-2.sh, proof: ac-2.log -->

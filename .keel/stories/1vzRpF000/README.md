---
id: 1vzRpF000
title: Route Hosted Service Lifecycle Through Live Runtime
type: feat
status: in-progress
created_at: 2026-03-08T21:02:37
updated_at: 2026-03-08T21:30:08
scope: 1vz4Yn000/1vzRnO000
started_at: 2026-03-08T21:30:08
---

# Route Hosted Service Lifecycle Through Live Runtime

## Summary

Turn hosted `port service apply|list|status|stop` into live execution through
the control plane, node agent, and guest runtime instead of desired-state-only
storage.

## Acceptance Criteria

<!-- verify: command, SRS-02:end, proof: ac-1.log -->
- [ ] [SRS-02/AC-01] The hosted runtime materializes stored machine secrets into launched guest processes, persists node-owned runtime state, and reports operator-safe log and exit metadata without surfacing raw secret values. <!-- [SRS-02/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzRpF000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-03:start:end, proof: ac-2.log -->
- [ ] [SRS-03/AC-02] Hosted `port service apply`, `list`, `status`, and `stop` execute and report real service or sandbox lifecycle state through the canonical CLI, SDK, and live hosted route instead of only mutating stored desired state. <!-- [SRS-03/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzRpF000/verify-ac-2.sh, proof: ac-2.log -->

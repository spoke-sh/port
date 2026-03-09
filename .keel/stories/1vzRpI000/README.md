---
id: 1vzRpI000
title: Publish Hosted Service And Sandbox Workflow
type: feat
status: backlog
created_at: 2026-03-08T21:02:40
updated_at: 2026-03-08T21:04:26
scope: 1vz4Yn000/1vzRnO000
---

# Publish Hosted Service And Sandbox Workflow

## Summary

Publish the hosted service and sandbox execution workflow so operators can
discover, learn, and verify what now runs live versus what still remains
follow-on work.

## Acceptance Criteria

<!-- verify: command, SRS-04:start, proof: ac-1.log -->
- [ ] [SRS-04/AC-01] CLI help, README, hosted docs, and SDK docs explain the hosted service and sandbox execution workflow through the canonical `port service` surface. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzRpI000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-04:end, proof: ac-2.log -->
- [ ] [SRS-04/AC-02] Published proof and operator messaging make the boundary explicit between shipped hosted execution and still-planned work such as restart policy, hardened secret backends, and scheduler policy. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzRpI000/verify-ac-2.sh, proof: ac-2.log -->

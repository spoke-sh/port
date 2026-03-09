---
id: 1vzUq9000
title: Publish Durable Hosted Fleet Workflow
type: feat
status: backlog
created_at: 2026-03-09T00:16:45
updated_at: 2026-03-09T00:20:39
scope: 1vzUnI000/1vzUoK000
---

# Publish Durable Hosted Fleet Workflow

## Summary

Publish the durable hosted fleet workflow through canonical CLI help, README,
hosted docs, and proof so operators can discover registration persistence,
heartbeat freshness, imported inventory, and the limits that remain after this
voyage.

## Acceptance Criteria

<!-- verify: command, SRS-05:start, proof: ac-1.log -->
- [ ] [SRS-05/AC-01] CLI help, README, and hosted docs publish the durable registration, heartbeat freshness, and imported inventory workflow through canonical `port machine`, `port control-plane`, and `port node-agent` surfaces. <!-- [SRS-05/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzUq9000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-05:end, proof: ac-2.log -->
- [ ] [SRS-05/AC-02] Repo-local proof covers restart recovery or imported-inventory inspection so operators can learn the workflow from executable evidence, not prose only. <!-- [SRS-05/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzUq9000/verify-ac-2.sh, proof: ac-2.log -->
<!-- verify: command, SRS-05:start, proof: ac-3.log -->
- [ ] [SRS-05/AC-03] The voyage closes with board evidence and verification on the implemented stories rather than leaving a second hosted planning-only backlog, satisfying `SRS-NFR-03`. <!-- [SRS-05/AC-03] verify: nix develop -c keel verify run 1vzUq9000, proof: ac-3.log -->
<!-- verify: command, SRS-05:end, proof: ac-3.log -->

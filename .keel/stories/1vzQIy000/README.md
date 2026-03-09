---
id: 1vzQIy000
title: Publish Hosted Detached Forward Workflow
type: feat
status: backlog
created_at: 2026-03-08T19:25:12
updated_at: 2026-03-08T19:27:01
scope: 1vzETR000/1vzQEj000
---

# Publish Hosted Detached Forward Workflow

## Summary

Publish the canonical hosted detached forward operator workflow across help
text, docs, and proof so the lifecycle commands are discoverable and runnable.

## Acceptance Criteria

<!-- verify: command, SRS-04:start:end, proof: ac-1.log -->
- [ ] [SRS-04/AC-01] CLI help, README, hosted docs, and SDK docs explain hosted detached `guest forward` start, list, stop, and `--name` behavior through the canonical Port surfaces. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzQIy000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-04:start:end, proof: ac-2.log -->
- [ ] [SRS-04/AC-02] The published workflow and proof make the hosted detached-forward boundary explicit enough that operators can tell what is shipped versus what remains follow-on work. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzQIy000/verify-ac-2.sh, proof: ac-2.log -->

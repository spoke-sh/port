---
id: 1vzEVX000
title: Publish Hosted Demo Workflow And Evidence
type: feat
status: backlog
created_at: 2026-03-08T06:49:23
updated_at: 2026-03-08T06:51:22
scope: 1vzETR000/1vzETX000
---

# Publish Hosted Demo Workflow And Evidence

## Summary

Publish the runnable hosted demo workflow, examples, and board evidence once the
control-plane and node-agent transport is live.

## Acceptance Criteria

<!-- verify: manual, SRS-05:start:end, proof: ac-1.log, ac-2.log -->
- [ ] [SRS-05/AC-01] README, hosted docs, and CLI help show how to start the control plane and node agent, then run canonical hosted machine and guest commands end-to-end. <!-- [SRS-05/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzEVX000/verify-ac-1.sh, proof: ac-1.log -->
- [ ] [SRS-05/AC-02] Repository-local evidence proves the hosted demo workflow is reproducible and clearly calls out any remaining transport limits. <!-- [SRS-05/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzEVX000/verify-ac-2.sh, proof: ac-2.log -->

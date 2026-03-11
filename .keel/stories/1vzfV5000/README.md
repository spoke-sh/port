---
id: 1vzfV5000
title: Publish Service Reliability Operator Workflow
type: feat
status: done
created_at: 2026-03-09T11:38:43
updated_at: 2026-03-11T11:49:35
scope: 1vzfT4000/1vzfTm000
started_at: 2026-03-11T11:40:14
completed_at: 2026-03-11T11:49:35
---

# Publish Service Reliability Operator Workflow

## Summary

Publish the shipped service-reliability workflow across the CLI, docs, sample
config, and recorded proofs so operators can discover and execute secret-backed
services with restart and health visibility through canonical Port commands.

## Acceptance Criteria

<!-- verify: command, SRS-04:start, proof: ac-1.log -->
- [x] [SRS-04/AC-01] README, hosted/operator docs, CLI help, and sample-config guidance publish the service reliability workflow and its remaining limits through the canonical `port service` surface. <!-- [SRS-04/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && bash scripts/verify-service-reliability-docs.sh', proof: ac-1.log -->
<!-- verify: command, SRS-04:end -->
<!-- verify: command, SRS-04:start, proof: ac-2.log -->
- [x] [SRS-04/AC-02] Port records a repo-local proof that stores a secret, launches a service, observes health or restart state, and stops the workload through canonical `port service` verbs. <!-- [SRS-04/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && bash scripts/service-reliability-demo.sh', proof: ac-2.log -->
<!-- verify: command, SRS-04:end -->

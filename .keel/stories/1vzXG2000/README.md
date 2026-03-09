---
id: 1vzXG2000
title: Define Hosted Standard Placement Contract
type: feat
status: in-progress
created_at: 2026-03-09T02:50:38
updated_at: 2026-03-09T02:57:25
scope: 1vzXFf000/1vzXFy000
started_at: 2026-03-09T02:57:25
---

# Define Hosted Standard Placement Contract

## Summary

Define the placement and routing contract for `standard` provider-backed hosted
machines so `cloud-generic`, `cloud-aws`, and `cloud-gcp` resolve onto explicit
registered nodes with actionable rejection detail instead of generic remote
unsupported-host guidance.

## Acceptance Criteria

<!-- verify: command, SRS-01:start, proof: ac-1.log -->
- [ ] [SRS-01/AC-01] Placement summary logic resolves candidate hosted nodes for `cloud-generic`, `cloud-aws`, and `cloud-gcp` while preserving machine, host, provider, and control-plane identity. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-model -p port-runtime hosted_standard', proof: ac-2.log -->
<!-- verify: command, SRS-01:continues, proof: ac-3.log -->
- [ ] [SRS-01/AC-02] Ineligible, unregistered, or unresolved standard-lane nodes fail with explicit routing context instead of the current generic “run Port on that host directly” provider guidance. <!-- [SRS-01/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_standard', proof: ac-2.log -->
<!-- verify: command, SRS-01:end, proof: ac-3.log -->
- [ ] [SRS-01/AC-03] The hosted placement contract serializes candidate-node and selected-node detail so later status or stop routes can follow the same provider-aware placement, satisfying `SRS-NFR-01`. <!-- [SRS-01/AC-03] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_standard', proof: ac-3.log -->

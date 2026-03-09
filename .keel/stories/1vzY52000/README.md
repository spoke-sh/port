---
id: 1vzY52000
title: Add Pvm Artifact Mobility Workflow
type: feat
status: backlog
created_at: 2026-03-09T03:43:20
updated_at: 2026-03-09T03:45:44
scope: 1vz3ck000/1vzY3z000
---

# Add Pvm Artifact Mobility Workflow

## Summary

Extend the canonical `port artifacts ...` surface so `x86_64/firecracker/pvm`
kernel and guest-image variants can be built, validated, pushed, and pulled
without implicit reuse of the standard Firecracker lane.

## Acceptance Criteria

<!-- verify: command, SRS-02:start -->
- [ ] [SRS-02/AC-01] `port artifacts build|validate|push|pull` supports the `x86_64/firecracker/pvm` kernel and guest-image variants through the canonical artifact model. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime artifact_variant && cargo test -q -p port-cli artifact', proof: ac-1.log -->
<!-- verify: command, SRS-02:end -->
<!-- verify: command, SRS-02:start -->
- [ ] [SRS-02/AC-02] PVM artifact mobility remains deterministic and explicit: missing variants fail with the selected variant name and Port does not fall back to standard Firecracker artifacts. <!-- [SRS-02/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime selected_variant && cargo test -q -p port-runtime pvm_host_kit_preflight', proof: ac-2.log -->
<!-- verify: command, SRS-02:end -->

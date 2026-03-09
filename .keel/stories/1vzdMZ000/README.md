---
id: 1vzdMZ000
title: Implement Local Cloud Hypervisor Machine Driver
type: feat
status: backlog
scope: 1vzdKB000/1vzdKx000
created_at: 2026-03-09T09:21:47
updated_at: 2026-03-09T09:25:33
---

# Implement Local Cloud Hypervisor Machine Driver

## Summary

Implement the local Cloud Hypervisor launch, status, and stop path through
Port's machine-driver seam, including runtime manifest ownership and console
capture.

## Acceptance Criteria

<!-- verify: command, SRS-02:start, proof: ac-1.log -->
- [ ] [SRS-02/AC-01] `port machine launch|status|stop` executes a Cloud Hypervisor machine locally through the canonical driver boundary and records coherent runtime metadata. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime -p port-cli --test machine_commands', proof: ac-1.log -->
<!-- verify: command, SRS-02:end -->
<!-- verify: command, SRS-02:start, proof: ac-2.log -->
- [ ] [SRS-02/AC-02] Local Cloud Hypervisor preflight failures identify the missing host prerequisite or runtime boundary instead of generic unsupported-host output. <!-- [SRS-02/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime -p port-cli', proof: ac-2.log -->
<!-- verify: command, SRS-02:end -->

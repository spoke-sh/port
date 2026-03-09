---
id: 1vzdMs000
title: Bridge Cloud Hypervisor Guest Sessions
type: feat
status: backlog
scope: 1vzdKB000/1vzdKx000
created_at: 2026-03-09T09:22:06
updated_at: 2026-03-09T09:25:33
---

# Bridge Cloud Hypervisor Guest Sessions

## Summary

Bridge Cloud Hypervisor guest transport onto Port's shared guest protocol so
guest exec, copy, pty, logs, and forward work without a substrate-specific API.

## Acceptance Criteria

<!-- verify: command, SRS-03:start, proof: ac-1.log -->
- [ ] [SRS-03/AC-01] Cloud Hypervisor machines expose guest `exec`, `copy`, `pty`, `logs`, and `forward` through the canonical Port guest protocol. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime -p port-guest-agent -p port-agent-protocol -p port-cli --test guest_commands', proof: ac-1.log -->
<!-- verify: command, SRS-03:end -->
<!-- verify: command, SRS-03:start, proof: ac-2.log -->
- [ ] [SRS-03/AC-02] The Cloud Hypervisor guest path reuses the existing protocol and hosted route families rather than inventing a second substrate-specific guest API. <!-- [SRS-03/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime -p port-agent-protocol -p port-sdk', proof: ac-2.log -->
<!-- verify: command, SRS-03:end -->

---
id: 1vzY6F000
title: Implement Hosted Pvm Node Preparation
type: feat
status: backlog
created_at: 2026-03-09T03:44:35
updated_at: 2026-03-09T03:45:44
scope: 1vz3ck000/1vzY3z000
---

# Implement Hosted Pvm Node Preparation

## Summary

Implement the hosted node-preparation/import flow that upgrades a node from
planned PVM capacity to ready PVM capacity when a complete host-kit package is
attached through canonical hosted inventory and node-agent state.

## Acceptance Criteria

<!-- verify: command, SRS-03:start -->
- [ ] [SRS-03/AC-01] Port can prepare or import a hosted node with a complete PVM host-kit package so hosted inventory records a ready `x86_64` Firecracker/PVM node instead of only planned capacity. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_pvm && cargo test -q -p port-cli machine_commands -- --exact', proof: ac-1.log -->
<!-- verify: command, SRS-03:end -->
<!-- verify: command, SRS-03:start -->
- [ ] [SRS-03/AC-02] Hosted placement and doctor output distinguish ready PVM nodes from planned or incomplete nodes with node-specific remediation guidance. <!-- [SRS-03/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_pvm_launch && cargo test -q -p port-cli machine_commands cli_hosted_pvm', proof: ac-2.log -->
<!-- verify: command, SRS-03:end -->

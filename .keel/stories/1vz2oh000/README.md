---
id: 1vz2oh000
title: Publish Hosted Node Agent Contract
type: feat
status: done
created_at: 2026-03-07T17:20:23
updated_at: 2026-03-07T17:56:14
scope: 1vz2eV000/1vz2ky000
started_at: 2026-03-07T17:52:47
submitted_at: 2026-03-07T17:56:07
completed_at: 2026-03-07T17:56:14
---

# Publish Hosted Node Agent Contract

## Summary

Define the first canonical hosted-Port contract: a node-local agent plus central
control plane that preserve today's guest-operation model while adding remote
lifecycle ownership, transport brokering, and inventory.

## Acceptance Criteria

<!-- verify: manual, SRS-04:start:end, proof: ac-1.log-->
- [x] [SRS-04/AC-01] Port publishes a canonical hosted-control document describing node-agent responsibilities, control-plane responsibilities, and machine lifecycle ownership for local versus hosted execution. <!-- [SRS-04/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "Node Agent|Control Plane|Lifecycle Ownership|Local Port today|Hosted Port planned" docs/hosted.md README.md', proof: ac-1.log-->
<!-- verify: manual, SRS-04:start:end, proof: ac-2.log-->
- [x] [SRS-04/AC-02] The contract explains how guest `exec`, `copy`, `pty`, `logs`, and `forward` are brokered through the hosted product without replacing the current guest protocol semantics. <!-- [SRS-04/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "exec|copy|pty|logs|forward|guest protocol|broker|connect_guest" docs/hosted.md', proof: ac-2.log-->
<!-- verify: manual, SRS-04:start:end, proof: ac-3.log-->
- [x] [SRS-04/AC-03] README and linked docs surface the hosted contract and the current support matrix so operators can distinguish shipped local behavior from planned hosted behavior. <!-- [SRS-04/AC-03] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && env CARGO_TARGET_DIR=/tmp/port-target cargo run -p port-cli -- --help >/tmp/1vz2oh000-help.verify && rg -n "Hosted Control Preview|docs/hosted.md|Hosted Control:" README.md docs/cloud.md docs/operators.md crates/port-cli/src/lib.rs', proof: ac-3.log-->

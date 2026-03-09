---
id: 1vzfVW000
title: Define Service Policy And Health Contract
type: feat
status: icebox
created_at: 2026-03-09T11:39:10
updated_at: 2026-03-09T11:39:32
scope: 1vzfT4000/1vzfTm000
---

# Define Service Policy And Health Contract

## Summary

Define the shared restart-policy and health-policy contract for Port services
and sandboxes, then thread that contract through the canonical CLI, hosted API,
and SDK surfaces without introducing a second service model.

## Acceptance Criteria

<!-- verify: command, SRS-01:start, proof: ac-1.log -->
- [ ] [SRS-01/AC-01] `port service` config, help, hosted request/status payloads, and SDK types expose the shared restart-policy and health-policy contract without adding a hosted-only alias. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-model service_policy && cargo test -q -p port-sdk service_policy && cargo test -q -p port-cli service_policy', proof: ac-1.log -->
<!-- verify: command, SRS-01:end -->
<!-- verify: command, SRS-NFR-02:start, proof: ac-2.log -->
- [ ] [SRS-NFR-02/AC-01] Unsupported restart-policy or health-policy combinations fail fast with explicit diagnostics and no fallback to legacy service behavior. <!-- [SRS-NFR-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-model service_policy_invalid && cargo test -q -p port-cli service_policy_invalid', proof: ac-2.log -->
<!-- verify: command, SRS-NFR-02:end -->

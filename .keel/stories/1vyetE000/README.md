---
id: 1vyetE000
title: Implement Remote Linux Diagnostics
type: feat
status: backlog
created_at: 2026-03-06T15:47:28
updated_at: 2026-03-06T15:48:36
scope: 1vydg7000/1vyeq5000
---

# Implement Remote Linux Diagnostics

## Summary

Teach the canonical CLI/runtime surfaces to understand remote Linux provider
intent, report support boundaries in `port doctor`, and fail fast with
actionable guidance when operators try to launch against unimplemented remote
cloud hosts.

## Acceptance Criteria

<!-- verify: manual, SRS-02:start:end, proof: ac-1.log-->
- [ ] [SRS-02/AC-01] `port doctor` emits provider-aware diagnostics for generic remote Linux, AWS, GCP, and Azure host targets without overstating implementation status. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -p port-runtime -p port-cli && nix develop -c cargo run -p port-cli -- --config /home/alex/workspace/spoke-sh/port/examples/port.toml doctor', proof: ac-2.log-->
<!-- verify: manual, SRS-03:start:end, proof: ac-3.log-->
- [ ] [SRS-03/AC-01] `port machine launch` rejects remote cloud hosts with provider-specific next-step guidance instead of a generic unsupported-host error. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -p port-runtime -p port-cli && nix develop -c cargo run -p port-cli -- --config /home/alex/workspace/spoke-sh/port/examples/port.toml machine launch --machine cloud-aws', proof: ac-4.log-->

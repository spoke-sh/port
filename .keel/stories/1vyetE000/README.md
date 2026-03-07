---
id: 1vyetE000
title: Implement Remote Linux Diagnostics
type: feat
status: done
created_at: 2026-03-06T15:47:28
updated_at: 2026-03-06T15:58:07
scope: 1vydg7000/1vyeq5000
started_at: 2026-03-06T15:54:34
submitted_at: 2026-03-06T15:57:58
completed_at: 2026-03-06T15:58:07
---

# Implement Remote Linux Diagnostics

## Summary

Teach the canonical CLI/runtime surfaces to understand remote Linux provider
intent, report support boundaries in `port doctor`, and fail fast with
actionable guidance when operators try to launch against unimplemented remote
cloud hosts.

## Acceptance Criteria

<!-- verify: manual, SRS-02:start:end, proof: ac-1.log-->
- [x] [SRS-02/AC-01] `port doctor` emits provider-aware diagnostics for generic remote Linux, AWS, GCP, and Azure host targets without overstating implementation status. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime -p port-cli && /tmp/port-target/debug/port --config examples/port.toml doctor', proof: ac-1.log-->
<!-- verify: manual, SRS-03:start:end, proof: ac-2.log-->
- [x] [SRS-03/AC-01] `port machine launch` rejects remote cloud hosts with provider-specific next-step guidance instead of a generic unsupported-host error. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && set +e; output=$(/tmp/port-target/debug/port --config examples/port.toml machine launch --machine cloud-aws 2>&1); status=$?; printf "%s\n" "$output"; test "$status" -eq 1; printf "%s\n" "$output" | rg -q "AWS remains a justified future Firecracker lane"; printf "%s\n" "$output" | rg -q "Run Port on the AWS Linux host itself"', proof: ac-2.log-->

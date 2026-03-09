---
id: 1vzeY9000
title: Implement OCI Artifact Push Transport
type: feat
status: done
created_at: 2026-03-09T10:37:49
updated_at: 2026-03-09T11:04:41
scope: 1vzW8e000/1vzeWr000
started_at: 2026-03-09T10:56:12
completed_at: 2026-03-09T11:04:41
---

# Implement OCI Artifact Push Transport

## Summary

Implement the runtime and CLI push path for selected artifact variants routed
through the new `oci-registry` backend, including backend detail reporting and
explicit failure context.

## Acceptance Criteria

<!-- verify: command, SRS-02:start, proof: ac-1.log -->
- [x] [SRS-02/AC-01] `port artifacts push` publishes the selected artifact variant through the `oci-registry` backend while preserving the existing artifact reference and selector vocabulary. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime oci_registry_push && cargo test -q -p port-cli --test artifact_commands cli_artifact_push_oci_registry', proof: ac-1.log -->
<!-- verify: command, SRS-02:end -->
<!-- verify: command, SRS-02:start, proof: ac-2.log -->
- [x] [SRS-02/AC-02] OCI push reports the resolved remote reference, selected variant, backend detail, cache path, and local path ownership while preserving the canonical artifact vocabulary. <!-- [SRS-02/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime oci_registry_push_failure && cargo test -q -p port-cli --test artifact_commands cli_artifact_push_oci_registry', proof: ac-2.log -->
<!-- verify: command, SRS-02:end -->

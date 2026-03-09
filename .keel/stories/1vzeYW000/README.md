---
id: 1vzeYW000
title: Implement OCI Artifact Pull Transport
type: feat
status: backlog
created_at: 2026-03-09T10:38:12
updated_at: 2026-03-09T10:43:29
scope: 1vzW8e000/1vzeWr000
---

# Implement OCI Artifact Pull Transport

## Summary

Implement the runtime and CLI pull path for selected artifact variants routed
through the new `oci-registry` backend, hydrating the canonical cache and local
artifact paths from the remote OCI reference and finalizing the shared transfer
reporting and failure context for the OCI backend.

## Acceptance Criteria

<!-- verify: command, SRS-03:start, proof: ac-1.log -->
- [ ] [SRS-03/AC-01] `port artifacts pull` fetches the selected artifact variant from the `oci-registry` backend into the canonical cache and local artifact paths without adding a second retrieval workflow. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime oci_registry_pull && cargo test -q -p port-cli --test artifact_commands cli_artifact_pull_oci_registry', proof: ac-1.log -->
<!-- verify: command, SRS-03:end -->
<!-- verify: command, SRS-03:start, proof: ac-2.log -->
- [ ] [SRS-03/AC-02] OCI pull preserves the same deterministic cache and local artifact paths used by the other distribution backends for the same artifact reference and selector, satisfying `SRS-NFR-01`. <!-- [SRS-03/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime oci_registry_cache_path', proof: ac-2.log -->
<!-- verify: command, SRS-03:end -->
<!-- verify: command, SRS-04:start, proof: ac-3.log -->
- [ ] [SRS-04/AC-01] OCI transfer failures and final runtime reporting surface the resolved remote reference, selected variant, auth source, backend detail, cache path, and local path ownership. <!-- [SRS-04/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime oci_registry_pull_failure && cargo test -q -p port-cli --test artifact_commands cli_artifact_pull_oci_registry', proof: ac-3.log -->
<!-- verify: command, SRS-04:end -->

---
id: 1vzeYV000
title: Define OCI Registry Artifact Contract
type: feat
status: backlog
created_at: 2026-03-09T10:38:11
updated_at: 2026-03-09T10:43:29
scope: 1vzW8e000/1vzeWr000
---

# Define OCI Registry Artifact Contract

## Summary

Define the canonical `oci-registry` artifact backend contract in the model,
doctor, and runtime backend resolver so Port can describe OCI transport,
auth-source, and prerequisite behavior as a real product lane instead of a
reserved runtime stub.

## Acceptance Criteria

<!-- verify: command, SRS-01:start, proof: ac-1.log -->
- [ ] [SRS-01/AC-01] Port defines a canonical `oci-registry` artifact-store contract with deterministic remote-reference derivation inputs, explicit auth sourcing, and explicit transport policy for selected variants. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-model oci_registry', proof: ac-1.log -->
<!-- verify: command, SRS-01:end -->
<!-- verify: command, SRS-01:start, proof: ac-2.log -->
- [ ] [SRS-01/AC-02] Doctor and runtime backend validation fail fast with explicit dependency or auth-source detail when an OCI backend is configured incorrectly, satisfying `SRS-NFR-03`, and they do not fall back to any other backend. <!-- [SRS-01/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime oci_registry_backend', proof: ac-2.log -->
<!-- verify: command, SRS-01:end -->

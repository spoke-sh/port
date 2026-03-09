---
id: 1vzWCI000
title: Implement Hosted Artifact Control Plane Routes
type: feat
status: done
created_at: 2026-03-09T01:42:42
updated_at: 2026-03-09T02:14:12
scope: 1vzW8e000/1vzW9Q000
started_at: 2026-03-09T02:00:16
completed_at: 2026-03-09T02:14:12
---

# Implement Hosted Artifact Control Plane Routes

## Summary

Implement authenticated hosted control-plane upload and download routes for one
selected artifact variant, backed by a deterministic control-plane-owned store
path under `.port/hosted/...`.

## Acceptance Criteria

<!-- verify: command, SRS-02:start, proof: ac-1.log -->
- [x] [SRS-02/AC-01] The hosted control plane exposes authenticated artifact upload and download routes that stream one selected artifact variant into and out of the control-plane-owned hosted store. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-hosted-protocol -p port-runtime', proof: ac-2.log -->
<!-- verify: command, SRS-02:continues, proof: ac-3.log -->
- [x] [SRS-02/AC-02] Upload and download handlers persist and locate hosted artifacts at the deterministic store path derived from artifact reference and selector, satisfying `SRS-NFR-01`. <!-- [SRS-02/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-hosted-protocol -p port-runtime', proof: ac-2.log -->
<!-- verify: command, SRS-02:end, proof: ac-3.log -->
- [x] [SRS-02/AC-03] Hosted route failures include artifact reference, selector, backend, and endpoint or store-path detail so operators get actionable error context, satisfying `SRS-NFR-02`. <!-- [SRS-02/AC-03] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-hosted-protocol -p port-runtime', proof: ac-3.log -->

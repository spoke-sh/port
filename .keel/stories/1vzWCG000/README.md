---
id: 1vzWCG000
title: Define Hosted Artifact Backend Contract
type: feat
status: backlog
created_at: 2026-03-09T01:42:40
updated_at: 2026-03-09T01:45:28
scope: 1vzW8e000/1vzW9Q000
---

# Define Hosted Artifact Backend Contract

## Summary

Define the executable hosted artifact backend contract across the shared model,
runtime selection logic, and hosted protocol so `ArtifactStore::HostedApi`
stops being modeled-only and resolves a deterministic store path for one
selected artifact variant.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] `port-model` and runtime helpers resolve `ArtifactStore::HostedApi { endpoint }` for a selected artifact reference and selector into deterministic backend metadata, including hosted endpoint, filename, and hosted store path.
- [ ] [SRS-01/AC-02] Validation fails fast when a hosted artifact backend is misconfigured or unsupported, and it does not silently fall back to the file-system backend, satisfying `SRS-NFR-02`.
- [ ] [SRS-01/AC-03] Shared hosted protocol contracts cover hosted artifact push and pull metadata, including artifact reference, selector, backend, and hosted store path or endpoint detail for success and failure paths.

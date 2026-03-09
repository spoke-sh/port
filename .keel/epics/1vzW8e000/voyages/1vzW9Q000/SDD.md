# Hosted Artifact Push And Pull - Software Design Description

> Implement the first live hosted-api artifact backend so canonical Port push and pull flows work end-to-end through the hosted control plane.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage extends Port's existing artifact selector model with one executable
remote backend: `hosted-api`. The operator still runs `port artifacts push|pull`
locally. Runtime resolves the selected artifact variant, authenticates to the
configured hosted control plane, and either uploads the variant bytes into a
control-plane-owned artifact store or downloads them back into the local output
path and cache root. The control plane owns the first shipped hosted store
under `.port/hosted/<control-plane>/artifacts/...`, which keeps the backend
compatible with the existing hosted auth contract and avoids inventing node or
machine affinity for a global artifact verb.

## Context & Boundaries

- In scope:
  - hosted control-plane transport and store ownership for artifact variants
  - shared hosted protocol contracts for artifact upload/download
  - CLI/runtime routing for `artifacts push|pull`
  - documentation and executable proof
- Out of scope:
  - OCI transport
  - node-agent-owned artifact storage
  - lifecycle management such as GC, quotas, or deduplication

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `port-model` artifact selector and `ArtifactStore::HostedApi` | internal | canonical artifact backend configuration | current workspace |
| `port-hosted-protocol` | internal | shared request and response contracts for hosted HTTP | current workspace |
| hosted bearer-token auth contract | internal | authenticate artifact transfer requests | current workspace |
| Rust unit tests and command proofs | verification | prove model/runtime/CLI behavior | current workspace |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Store ownership | control-plane-owned filesystem store under `.port/hosted/<control-plane>/artifacts/...` | `artifacts push|pull` are global artifact verbs, not node-scoped lifecycle actions. |
| Transport shape | authenticated hosted HTTP routes with selector metadata plus streamed binary bodies | Reuses the existing hosted product path without inventing an internal artifact-only daemon. |
| Selector contract | keep `architecture`, `substrate`, and `protection_mode` exactly as today | Avoids a second artifact compatibility model. |
| Backend rollout | ship `hosted-api` first and leave OCI explicit follow-on work | One live remote backend is higher value than two partial ones. |

## Architecture

The voyage adds three layers:

1. Model/runtime backend resolution:
   - resolve `ArtifactStore::HostedApi { endpoint }`
   - derive deterministic hosted store path from artifact reference plus selector
   - reuse existing cache and local output paths
2. Hosted transport:
   - extend `port-hosted-protocol` with artifact push/pull request metadata
   - add control-plane handlers for upload/download
   - authenticate with the existing hosted bearer token contract
3. CLI surface:
   - `port artifacts push|pull` dispatch to hosted runtime helpers
   - output includes backend, selector, cache path, and hosted store path

## Components

### `port-model`

- Keeps `ArtifactStore::HostedApi { endpoint }` as the canonical config token.
- Adds helpers for deterministic hosted store-path resolution and validation.

### `port-hosted-protocol`

- Defines typed artifact transfer metadata shared by client and server.
- Carries artifact reference, selector, filename, and control-plane store-path
  context in success and failure responses.

### `port-runtime`

- Extends artifact transfer helpers to branch on `ArtifactStore::HostedApi`.
- Implements hosted client calls for push and pull.
- Adds control-plane server handlers that persist bytes under the hosted store
  root and stream them back on pull.

### `port-cli`

- Leaves the command family unchanged.
- Prints hosted backend details and proof-friendly output for push and pull.

## Interfaces

- CLI:
  - `port artifacts push --artifact <name> [selectors...]`
  - `port artifacts pull --artifact <name> [selectors...]`
- Hosted transport:
  - authenticated artifact upload route
  - authenticated artifact download route
- Filesystem:
  - local output path remains the canonical materialized artifact location
  - cache path remains `.port/cache/...`
  - hosted store path becomes `.port/hosted/<control-plane>/artifacts/<registry>/<repository>/<version>/<architecture>/<substrate>/<protection-mode>/<filename>`

## Data Flow

### Push

1. CLI resolves artifact reference plus selected variant.
2. Runtime validates local artifact path exists and resolves backend.
3. For `hosted-api`, runtime authenticates to the configured control plane.
4. Control plane streams the uploaded bytes into the deterministic hosted store
   path and returns transfer metadata.
5. CLI prints local path, cache path, backend, and hosted store path.

### Pull

1. CLI resolves artifact reference plus selected variant.
2. Runtime authenticates to the hosted control plane and requests the selected
   variant.
3. Control plane locates the deterministic hosted store path and streams bytes
   back to the client.
4. Runtime writes bytes into both the local output path and cache path.
5. CLI prints backend, hosted store path, local path, and cache path.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Hosted backend configured but unsupported | runtime backend match or validation failure | fail fast with backend and artifact detail; no file-system fallback | correct config or implement backend |
| Artifact variant missing locally on push | local path missing | fail with artifact reference, selector, and local path | build or pull the variant first |
| Hosted store entry missing on pull | control-plane store lookup fails | fail with artifact reference, selector, backend, and hosted store path | push the variant first or repair store |
| Auth token missing or rejected | hosted auth contract resolution or HTTP response | fail with endpoint and auth-source context | provide token or repair auth configuration |
| Selector mismatch or unsupported variant | model/runtime resolution fails | fail with artifact reference plus selector detail | choose a supported variant or add it explicitly |

# OCI Artifact Registry Mobility - Software Design Description

> Ship the reserved oci-registry backend through canonical artifact push and pull workflows with explicit auth, path, and failure contracts.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage turns the reserved `oci-registry` token into a real artifact
backend. Port keeps the existing artifact reference plus selector model, derives
one deterministic OCI remote reference per selected variant, shells out to a
single registry client for push and pull, and keeps the canonical local/cache
paths unchanged. Doctor, help text, examples, and helper tasks make the backend
discoverable and explicit about prerequisites.

## Context & Boundaries

```
┌────────────────────────────────────────────────────────────────┐
│                           This Voyage                          │
│                                                                │
│  port model  ->  runtime backend resolver  ->  oras adapter    │
│      │                   │                         │            │
│      └──── doctor/help --┴---- CLI push/pull -----┘            │
└────────────────────────────────────────────────────────────────┘
                          │
                    [OCI registry]
```

In scope:
- artifact-store contract changes for `oci-registry`
- runtime push and pull execution
- doctor/help/example/just workflow publication

Out of scope:
- in-process registry transport
- signing, SBOMs, manifest lists, and deduplication
- backend-specific CLI verbs

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `oras` | CLI tool | Push and pull selected artifact files to or from OCI registry references. | CLI on `PATH` |
| `zot` | local registry process | Repo-local proof and helper-task registry target. | local dev proof only |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| OCI transport implementation | Shell out to `oras` for the first slice | Port already exposes binary prerequisites through `doctor`; this keeps the implementation bounded while shipping a real backend quickly. |
| Remote reference derivation | Derive one OCI reference from canonical artifact identity plus selector | This preserves one source of truth for artifact naming and avoids adding backend-specific names to the model. |
| Auth contract | Support anonymous and env-backed basic auth in the first slice | This is sufficient for local proofs and common private registries without storing secrets in the model. |
| Local registry proof | Use a repo-local `zot` registry plus helper tasks | The proof stays local, repeatable, and independent of public registries. |

## Architecture

1. `port-model`
   - replace the reserved `OciRegistry { reference }` stub with a real contract
     that carries transport and auth information
   - derive deterministic OCI remote references from `ArtifactReference` plus
     `ArtifactSelector`
2. `port-runtime`
   - extend backend resolution and doctor checks for OCI prerequisites
   - implement push and pull via a small command-adapter layer around `oras`
   - preserve current `ArtifactTransfer` local/cache reporting
3. `port-cli` and docs
   - keep `artifacts build|validate|push|pull`
   - publish examples, help text, and proof commands for local registry usage
4. helper workflows
   - add `just` tasks to start/stop a repo-local registry and run the proof

## Components

### Model contract

- Add an explicit OCI backend contract with:
  - transport mode, including plain HTTP for local registry proofs
  - auth source, initially `anonymous` or env-backed basic auth
- Keep artifact identity in `ArtifactReference`; do not duplicate repository or
  version strings in a second backend-specific identity field.

### Runtime adapter

- Build the OCI remote reference from:
  - `artifact.reference.registry`
  - `artifact.reference.repository`
  - `artifact.reference.version`
  - selector suffix for `architecture`, `substrate`, and `protection_mode`
- Execute `oras push` and `oras pull` through a small adapter that can be faked
  in unit tests.
- Normalize failures so Port reports backend, remote reference, auth source,
  and command context instead of raw shell output alone.

### Operator surface

- `port artifacts push|pull` print:
  - artifact name and selector
  - backend detail
  - remote OCI reference
  - cache path
  - local path
- `port doctor` checks the configured OCI backend prerequisites when present.
- `README.md`, `docs/artifacts.md`, `examples/port.toml`, and CLI help publish
  the OCI workflow and remove the old “follow-on work” wording for that lane.

## Interfaces

Model surface:
- `ArtifactStore::OciRegistry` becomes a real contract rather than a stub.

Runtime surface:
- `push_artifact()` and `pull_artifact()` resolve an `ArtifactStoreContract`
  that now includes OCI transport metadata.

Shell interface:
- `oras push <remote-ref> <file>:<media-type>`
- `oras pull <remote-ref> --output <temp-dir>`

Helper-task surface:
- `just oci-registry-up`
- `just oci-registry-down`
- `just demo-push-oci`
- `just demo-pull-oci`

## Data Flow

Push:
1. CLI resolves artifact and selector.
2. Runtime resolves backend contract and validates OCI prerequisites.
3. Runtime derives remote OCI reference.
4. Runtime shells out to `oras push`.
5. Runtime copies the same artifact to the canonical cache path.
6. CLI prints backend detail, remote reference, cache path, and local path.

Pull:
1. CLI resolves artifact and selector.
2. Runtime resolves backend contract and validates OCI prerequisites.
3. Runtime derives remote OCI reference.
4. Runtime shells out to `oras pull` into a temp directory.
5. Runtime copies the pulled file into the canonical cache path and artifact
   path.
6. CLI prints backend detail, remote reference, cache path, and local path.

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| `oras` missing from `PATH` | doctor check or command spawn failure | Fail fast with explicit OCI backend and binary detail | Install `oras` or use a shell that provides it |
| required auth env var missing | backend validation before spawn | Fail fast with auth-source detail and no fallback | Export the configured env vars |
| registry rejects push or pull | non-zero `oras` exit | Surface remote reference, selector, backend, and stderr context | Fix auth, transport, or registry availability and rerun |
| pulled artifact file missing from temp output | post-pull file inspection | Fail with remote reference and expected filename detail | Fix registry content or remote-reference derivation |

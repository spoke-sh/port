---
created_at: 2026-03-09T11:34:31
---

# Knowledge - 1vzeWr000

> Automated synthesis of story reflections.

## Story Knowledge

## Story: Implement OCI Artifact Push Transport (1vzeY9000)

### 1vzeY9000: Preserve canonical artifact references in backend transport tests

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | When adding a new artifact mobility backend that derives a backend-specific remote path from the canonical artifact reference. |
| **Insight** | Tests should vary the backend contract and transport behavior without mutating the canonical artifact reference, otherwise they stop verifying the user-facing vocabulary guarantees that the CLI promises. |
| **Suggested Action** | Keep `ArtifactReference` stable in CLI/runtime proofs and assert backend-specific remote paths separately through `store_path` or backend-detail output. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, `crates/port-cli/tests/artifact_commands.rs`, future artifact mobility stories |
| **Applied** | yes |



---

## Story: Implement OCI Artifact Pull Transport (1vzeYW000)

### 1vzeYW000: Keep OCI pull on the same artifact path contract as every other backend

| Field | Value |
|-------|-------|
| **Category** | artifact-mobility |
| **Context** | When adding a remote pull backend that downloads artifact bytes before materializing them into the workspace-local artifact layout. |
| **Insight** | Pull backends should use backend-private staging and then hydrate the canonical cache and local artifact paths, otherwise each backend grows its own retrieval layout and the CLI stops being predictable. |
| **Suggested Action** | Keep staging directories internal to the adapter and add a path-parity proof whenever a new artifact distribution backend is introduced. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, `crates/port-cli/tests/artifact_commands.rs`, future artifact mobility stories |
| **Applied** | yes |



---

## Story: Publish OCI Artifact Operator Workflow (1vzeYA000)

### 1vzfPb000: Keel Story Evidence Commands Need An Explicit Repo Root

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Recording story proof with `keel story record --cmd` for repo-scoped commands |
| **Insight** | `keel story record` does not guarantee execution from the repository root, so relative-path proof commands can fail even when the same command succeeds interactively from the shell |
| **Suggested Action** | Wrap repo-scoped proof commands in `bash -lc "cd /repo/root && ..."` or use absolute paths |
| **Applies To** | `.keel/stories/*`, `keel story record`, repo-local verification commands |
| **Applied** | AC-01 and AC-02 evidence for this story use an explicit `cd /home/alex/workspace/spoke-sh/port && ...` wrapper |



---

## Story: Define OCI Registry Artifact Contract (1vzeYV000)

### 1vzeZ1000: Derive OCI Variant References In The Model

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When Port adds a new artifact backend that still uses the canonical artifact reference and selector model |
| **Insight** | Putting variant-specific OCI reference derivation on `ArtifactReference` keeps backend naming deterministic and prevents the runtime, CLI, and docs from drifting into separate naming rules |
| **Suggested Action** | Reuse the model helper from every future OCI push, pull, and docs surface instead of rebuilding remote-reference strings in runtime code |
| **Applies To** | `crates/port-model/src/lib.rs`, `crates/port-runtime/src/lib.rs`, future OCI docs/help text |
| **Applied** | yes |



---

## Synthesis

### yZ7OFKClg: Preserve canonical artifact references in backend transport tests

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | When adding a new artifact mobility backend that derives a backend-specific remote path from the canonical artifact reference. |
| **Insight** | Tests should vary the backend contract and transport behavior without mutating the canonical artifact reference, otherwise they stop verifying the user-facing vocabulary guarantees that the CLI promises. |
| **Suggested Action** | Keep `ArtifactReference` stable in CLI/runtime proofs and assert backend-specific remote paths separately through `store_path` or backend-detail output. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, `crates/port-cli/tests/artifact_commands.rs`, future artifact mobility stories |
| **Linked Knowledge IDs** | 1vzeY9000 |
| **Score** | 0.82 |
| **Confidence** | 0.94 |
| **Applied** | yes |

### WQxHqPJ2M: Keep OCI pull on the same artifact path contract as every other backend

| Field | Value |
|-------|-------|
| **Category** | artifact-mobility |
| **Context** | When adding a remote pull backend that downloads artifact bytes before materializing them into the workspace-local artifact layout. |
| **Insight** | Pull backends should use backend-private staging and then hydrate the canonical cache and local artifact paths, otherwise each backend grows its own retrieval layout and the CLI stops being predictable. |
| **Suggested Action** | Keep staging directories internal to the adapter and add a path-parity proof whenever a new artifact distribution backend is introduced. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, `crates/port-cli/tests/artifact_commands.rs`, future artifact mobility stories |
| **Linked Knowledge IDs** | 1vzeYW000 |
| **Score** | 0.83 |
| **Confidence** | 0.95 |
| **Applied** | yes |

### I17J7u6fh: Keel Story Evidence Commands Need An Explicit Repo Root

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Recording story proof with `keel story record --cmd` for repo-scoped commands |
| **Insight** | `keel story record` does not guarantee execution from the repository root, so relative-path proof commands can fail even when the same command succeeds interactively from the shell |
| **Suggested Action** | Wrap repo-scoped proof commands in `bash -lc "cd /repo/root && ..."` or use absolute paths |
| **Applies To** | `.keel/stories/*`, `keel story record`, repo-local verification commands |
| **Linked Knowledge IDs** | 1vzfPb000 |
| **Score** | 0.67 |
| **Confidence** | 0.93 |
| **Applied** | AC-01 and AC-02 evidence for this story use an explicit `cd /home/alex/workspace/spoke-sh/port && ...` wrapper |

### y3m4n3T4j: Derive OCI Variant References In The Model

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When Port adds a new artifact backend that still uses the canonical artifact reference and selector model |
| **Insight** | Putting variant-specific OCI reference derivation on `ArtifactReference` keeps backend naming deterministic and prevents the runtime, CLI, and docs from drifting into separate naming rules |
| **Suggested Action** | Reuse the model helper from every future OCI push, pull, and docs surface instead of rebuilding remote-reference strings in runtime code |
| **Applies To** | `crates/port-model/src/lib.rs`, `crates/port-runtime/src/lib.rs`, future OCI docs/help text |
| **Linked Knowledge IDs** | 1vzeZ1000 |
| **Score** | 0.83 |
| **Confidence** | 0.93 |
| **Applied** | yes |


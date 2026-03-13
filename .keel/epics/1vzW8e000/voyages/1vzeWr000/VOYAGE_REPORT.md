# VOYAGE REPORT: OCI Artifact Registry Mobility

## Voyage Metadata
- **ID:** 1vzeWr000
- **Epic:** 1vzW8e000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 4/4 stories complete

## Implementation Narrative
### Implement OCI Artifact Push Transport
- **ID:** 1vzeY9000
- **Status:** done

#### Summary
Implement the runtime and CLI push path for selected artifact variants routed
through the new `oci-registry` backend, including backend detail reporting and
explicit failure context.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] `port artifacts push` publishes the selected artifact variant through the `oci-registry` backend while preserving the existing artifact reference and selector vocabulary. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime oci_registry_push && cargo test -q -p port-cli --test artifact_commands cli_artifact_push_oci_registry', proof: ac-1.log -->
- [x] [SRS-02/AC-02] OCI push reports the resolved remote reference, selected variant, backend detail, cache path, and local path ownership while preserving the canonical artifact vocabulary. <!-- [SRS-02/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime oci_registry_push_failure && cargo test -q -p port-cli --test artifact_commands cli_artifact_push_oci_registry', proof: ac-2.log -->

#### Implementation Insights
- **1vzeY9000: Preserve canonical artifact references in backend transport tests**
  - Insight: Tests should vary the backend contract and transport behavior without mutating the canonical artifact reference, otherwise they stop verifying the user-facing vocabulary guarantees that the CLI promises.
  - Suggested Action: Keep `ArtifactReference` stable in CLI/runtime proofs and assert backend-specific remote paths separately through `store_path` or backend-detail output.
  - Applies To: `crates/port-runtime/src/lib.rs`, `crates/port-cli/tests/artifact_commands.rs`, future artifact mobility stories
  - Category: testing


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzeY9000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzeY9000/EVIDENCE/ac-2.log)

### Publish OCI Artifact Operator Workflow
- **ID:** 1vzeYA000
- **Status:** done

#### Summary
Publish the shipped OCI artifact workflow across the CLI, docs, examples, and
helper tasks so operators can discover and execute a local registry
build/push/remove/pull proof without leaving Port’s canonical artifact surface.

#### Acceptance Criteria
- [x] [SRS-05/AC-01] README, `docs/artifacts.md`, CLI help, sample-config guidance, and helper tasks publish the executable local OCI registry workflow and its prerequisites. <!-- [SRS-05/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "oci-registry|oras|zot|demo-push-oci|demo-pull-oci" README.md docs/artifacts.md examples/port.toml justfile crates/port-cli/src/lib.rs', proof: ac-1.log -->
- [x] [SRS-05/AC-02] Port records a repo-local proof that builds a variant, pushes it to a local OCI registry, removes the local artifact copy, and pulls it back without depending on a public registry, satisfying `SRS-NFR-02`. <!-- [SRS-05/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && just demo-push-oci && just demo-pull-oci', proof: ac-2.log -->

#### Implementation Insights
- **1vzfPb000: Keel Story Evidence Commands Need An Explicit Repo Root**
  - Insight: `keel story record` does not guarantee execution from the repository root, so relative-path proof commands can fail even when the same command succeeds interactively from the shell
  - Suggested Action: Wrap repo-scoped proof commands in `bash -lc "cd /repo/root && ..."` or use absolute paths
  - Applies To: `.keel/stories/*`, `keel story record`, repo-local verification commands
  - Category: process


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzeYA000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzeYA000/EVIDENCE/ac-2.log)

### Define OCI Registry Artifact Contract
- **ID:** 1vzeYV000
- **Status:** done

#### Summary
Define the canonical `oci-registry` artifact backend contract in the model,
doctor, and runtime backend resolver so Port can describe OCI transport,
auth-source, and prerequisite behavior as a real product lane instead of a
reserved runtime stub.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Port defines a canonical `oci-registry` artifact-store contract with deterministic remote-reference derivation inputs, explicit auth sourcing, and explicit transport policy for selected variants. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-model oci_registry', proof: ac-1.log -->
- [x] [SRS-01/AC-02] Doctor and runtime backend validation fail fast with explicit dependency or auth-source detail when an OCI backend is configured incorrectly, satisfying `SRS-NFR-03`, and they do not fall back to any other backend. <!-- [SRS-01/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime oci_registry_backend', proof: ac-2.log -->

#### Implementation Insights
- **1vzeZ1000: Derive OCI Variant References In The Model**
  - Insight: Putting variant-specific OCI reference derivation on `ArtifactReference` keeps backend naming deterministic and prevents the runtime, CLI, and docs from drifting into separate naming rules
  - Suggested Action: Reuse the model helper from every future OCI push, pull, and docs surface instead of rebuilding remote-reference strings in runtime code
  - Applies To: `crates/port-model/src/lib.rs`, `crates/port-runtime/src/lib.rs`, future OCI docs/help text
  - Category: architecture


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzeYV000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzeYV000/EVIDENCE/ac-2.log)

### Implement OCI Artifact Pull Transport
- **ID:** 1vzeYW000
- **Status:** done

#### Summary
Implement the runtime and CLI pull path for selected artifact variants routed
through the new `oci-registry` backend, hydrating the canonical cache and local
artifact paths from the remote OCI reference and finalizing the shared transfer
reporting and failure context for the OCI backend.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] `port artifacts pull` fetches the selected artifact variant from the `oci-registry` backend into the canonical cache and local artifact paths without adding a second retrieval workflow. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime oci_registry_pull && cargo test -q -p port-cli --test artifact_commands cli_artifact_pull_oci_registry', proof: ac-1.log -->
- [x] [SRS-03/AC-02] OCI pull preserves the same deterministic cache and local artifact paths used by the other distribution backends for the same artifact reference and selector, satisfying `SRS-NFR-01`. <!-- [SRS-03/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime oci_registry_cache_path', proof: ac-2.log -->
- [x] [SRS-04/AC-01] OCI transfer failures and final runtime reporting surface the resolved remote reference, selected variant, auth source, backend detail, cache path, and local path ownership. <!-- [SRS-04/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime oci_registry_pull_failure && cargo test -q -p port-cli --test artifact_commands cli_artifact_pull_oci_registry', proof: ac-3.log -->

#### Implementation Insights
- **1vzeYW000: Keep OCI pull on the same artifact path contract as every other backend**
  - Insight: Pull backends should use backend-private staging and then hydrate the canonical cache and local artifact paths, otherwise each backend grows its own retrieval layout and the CLI stops being predictable.
  - Suggested Action: Keep staging directories internal to the adapter and add a path-parity proof whenever a new artifact distribution backend is introduced.
  - Applies To: `crates/port-runtime/src/lib.rs`, `crates/port-cli/tests/artifact_commands.rs`, future artifact mobility stories
  - Category: artifact-mobility


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzeYW000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzeYW000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vzeYW000/EVIDENCE/ac-3.log)



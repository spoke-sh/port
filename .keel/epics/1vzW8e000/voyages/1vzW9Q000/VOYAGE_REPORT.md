# VOYAGE REPORT: Hosted Artifact Push And Pull

## Voyage Metadata
- **ID:** 1vzW9Q000
- **Epic:** 1vzW8e000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 4/4 stories complete

## Implementation Narrative
### Define Hosted Artifact Backend Contract
- **ID:** 1vzWCG000
- **Status:** done

#### Summary
Define the executable hosted artifact backend contract across the shared model,
runtime selection logic, and hosted protocol so `ArtifactStore::HostedApi`
stops being modeled-only and resolves a deterministic store path for one
selected artifact variant.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `port-model` and runtime helpers resolve `ArtifactStore::HostedApi { endpoint }` for a selected artifact reference and selector into deterministic backend metadata, including hosted endpoint, filename, and hosted store path. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-model -p port-hosted-protocol -p port-runtime', proof: ac-2.log -->
- [x] [SRS-01/AC-02] Validation fails fast when a hosted artifact backend is misconfigured or unsupported, and it does not silently fall back to the file-system backend, satisfying `SRS-NFR-02`. <!-- [SRS-01/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-model -p port-hosted-protocol -p port-runtime', proof: ac-2.log -->
- [x] [SRS-01/AC-03] Shared hosted protocol contracts cover hosted artifact push and pull metadata, including artifact reference, selector, backend, and hosted store path or endpoint detail for success and failure paths. <!-- [SRS-01/AC-03] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-model -p port-hosted-protocol -p port-runtime', proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzWCG000/EVIDENCE/ac-1.log)
- [ac-3.log](../../../../stories/1vzWCG000/EVIDENCE/ac-3.log)
- [ac-2.log](../../../../stories/1vzWCG000/EVIDENCE/ac-2.log)

### Implement Hosted Artifact Control Plane Routes
- **ID:** 1vzWCI000
- **Status:** done

#### Summary
Implement authenticated hosted control-plane upload and download routes for one
selected artifact variant, backed by a deterministic control-plane-owned store
path under `.port/hosted/...`.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] The hosted control plane exposes authenticated artifact upload and download routes that stream one selected artifact variant into and out of the control-plane-owned hosted store. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-hosted-protocol -p port-runtime', proof: ac-2.log -->
- [x] [SRS-02/AC-02] Upload and download handlers persist and locate hosted artifacts at the deterministic store path derived from artifact reference and selector, satisfying `SRS-NFR-01`. <!-- [SRS-02/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-hosted-protocol -p port-runtime', proof: ac-2.log -->
- [x] [SRS-02/AC-03] Hosted route failures include artifact reference, selector, backend, and endpoint or store-path detail so operators get actionable error context, satisfying `SRS-NFR-02`. <!-- [SRS-02/AC-03] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-hosted-protocol -p port-runtime', proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzWCI000/EVIDENCE/ac-1.log)
- [ac-3.log](../../../../stories/1vzWCI000/EVIDENCE/ac-3.log)
- [ac-2.log](../../../../stories/1vzWCI000/EVIDENCE/ac-2.log)

### Route Artifact Push And Pull Through Hosted Backend
- **ID:** 1vzWCJ000
- **Status:** done

#### Summary
Route the canonical `port artifacts push|pull` commands through the hosted
artifact backend so operators use the existing CLI vocabulary while Port prints
deterministic backend and path details for the selected variant.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] `port artifacts push` routes to the configured hosted backend and uploads the selected artifact variant through the hosted transport instead of the file-system backend. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime push_and_pull_artifact_round_trip_through_live_hosted_backend && cargo test -q -p port-cli --test artifact_commands cli_artifact_push_and_pull_round_trip_through_hosted_backend', proof: ac-1.log -->
- [x] [SRS-03/AC-02] `port artifacts pull` routes to the hosted backend and materializes the selected variant into both the canonical local output path and the cache path. <!-- [SRS-03/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime push_and_pull_artifact_round_trip_through_live_hosted_backend && cargo test -q -p port-cli --test artifact_commands cli_artifact_push_and_pull_round_trip_through_hosted_backend', proof: ac-2.log -->
- [x] [SRS-03/AC-03] Canonical CLI output for hosted push and pull includes artifact selector, backend, local path, cache path, and hosted store path detail without introducing a second artifact command family. <!-- [SRS-03/AC-03] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-cli --test artifact_commands cli_artifact_push_and_pull_round_trip_through_hosted_backend', proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzWCJ000/EVIDENCE/ac-1.log)
- [ac-3.log](../../../../stories/1vzWCJ000/EVIDENCE/ac-3.log)
- [ac-2.log](../../../../stories/1vzWCJ000/EVIDENCE/ac-2.log)

### Publish Hosted Artifact Mobility Workflow
- **ID:** 1vzWCK000
- **Status:** done

#### Summary
Publish the first hosted artifact mobility workflow through README, artifact
docs, CLI help, and executable proof so operators can build, push, remove, and
pull a selected artifact variant end-to-end while understanding that OCI
support remains follow-on work.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] Repo-local proof builds a selected artifact variant, pushes it to the hosted backend, removes the local output, then pulls the same variant back successfully through the canonical CLI. <!-- [SRS-04/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-cli --test artifact_commands cli_artifact_build_push_and_pull_round_trip_through_hosted_backend -- --exact', proof: ac-2.log -->
- [x] [SRS-05/AC-01] README, `docs/artifacts.md`, and relevant CLI help publish the hosted artifact workflow, control-plane store ownership, and auth expectations while explicitly stating that OCI remains follow-on work. <!-- [SRS-05/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-cli tests::help_includes_primary_surfaces -- --exact && rg -n hosted-api README.md docs/artifacts.md crates/port-cli/src/lib.rs && rg -n PORT_DEMO_TOKEN README.md docs/artifacts.md crates/port-cli/src/lib.rs && rg -n follow-on README.md docs/artifacts.md crates/port-cli/src/lib.rs && rg -n .port/hosted README.md docs/artifacts.md crates/port-cli/src/lib.rs', proof: ac-2.log -->
- [x] [SRS-05/AC-02] The voyage closes with recorded board evidence and verification for the shipped hosted backend rather than leaving `hosted-api` as a modeled-only placeholder. <!-- [SRS-05/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime -p port-cli && keel doctor && test -f .keel/stories/1vzWCK000/EVIDENCE/ac-1.log && test -f .keel/stories/1vzWCK000/EVIDENCE/ac-2.log && test -f .keel/stories/1vzWCK000/EVIDENCE/ac-3.log', proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzWCK000/EVIDENCE/ac-1.log)
- [ac-3.log](../../../../stories/1vzWCK000/EVIDENCE/ac-3.log)
- [ac-2.log](../../../../stories/1vzWCK000/EVIDENCE/ac-2.log)



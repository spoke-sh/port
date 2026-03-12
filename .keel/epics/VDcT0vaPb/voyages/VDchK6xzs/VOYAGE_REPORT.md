# VOYAGE REPORT: Release Matrix And Packaging Foundations

## Voyage Metadata
- **ID:** VDchK6xzs
- **Epic:** VDcT0vaPb
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 4/4 stories complete

## Implementation Narrative
### Implement Canonical CLI Package Workflow
- **ID:** VDchsvWfn
- **Status:** done

#### Summary
Add the first canonical package build workflow for `port`, including a stable
artifact format, deterministic staging layout, and explicit failure guidance
for unsupported targets or missing prerequisites.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] The repo provides a canonical package workflow that emits one versioned install artifact per supported target with explicit target and included-file reporting. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q package_workflow', proof: ac-2.log -->
- [x] [SRS-NFR-01/AC-02] Package names, staging layout, and included files are deterministic across repeated runs for the same supported target. <!-- [SRS-NFR-01/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q package_determinism', proof: ac-2.log -->
- [x] [SRS-NFR-02/AC-03] Unsupported targets and missing packaging prerequisites fail fast with explicit guidance and no fallback to a source-only workflow. <!-- [SRS-NFR-02/AC-03] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q package_failure', proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VDchsvWfn/EVIDENCE/ac-1.log)
- [ac-3.log](../../../../stories/VDchsvWfn/EVIDENCE/ac-3.log)
- [ac-2.log](../../../../stories/VDchsvWfn/EVIDENCE/ac-2.log)

### Surface AVF Distribution Boundary In Docs And Doctor
- **ID:** VDchsw9fh
- **Status:** done

#### Summary
Align the install docs and doctor/help surfaces with the real AVF runtime
contract so macOS operators get explicit launcher-helper, entitlement, and
unsupported-host guidance as part of the packaged Port experience.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] macOS release guidance and doctor/help surfaces explain the `PORT_AVF_LAUNCHER` requirement, the launcher-helper role, the distributed-target entitlement boundary, and the expected unsupported-host guidance. <!-- [SRS-04/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime avf && rg -n "PORT_AVF_LAUNCHER|entitlement|distributed" README.md RELEASE.md docs/avf.md', proof: ac-1.log -->
- [x] [SRS-05/AC-02] The release checklist for the installable slice anchors validation on `just`, `port doctor`, workspace tests, package proof commands, and board health instead of a disconnected packaging-only checklist. <!-- [SRS-05/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "just mission|just doctor|just test|port doctor|package-proof|just package" README.md RELEASE.md justfile .justfiles', proof: ac-2.log -->
- [x] [SRS-NFR-02/AC-03] AVF packaging guidance fails fast with explicit boundaries and does not introduce a second macOS-only operator workflow or fallback surface. <!-- [SRS-NFR-02/AC-03] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-cli machine_commands && rg -n "macOS-only|launcher-helper|fallback" docs/avf.md README.md RELEASE.md', proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VDchsw9fh/EVIDENCE/ac-1.log)
- [ac-3.log](../../../../stories/VDchsw9fh/EVIDENCE/ac-3.log)
- [ac-2.log](../../../../stories/VDchsw9fh/EVIDENCE/ac-2.log)

### Add Install Proof For Packaged Port
- **ID:** VDchsxHfm
- **Status:** done

#### Summary
Prove that a packaged Port artifact can be extracted or installed into a clean
prefix and used through the canonical binary path without relying on repo-local
Cargo commands or external release infrastructure.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] The install proof extracts or installs the packaged artifact and runs the packaged `port` binary successfully for `--version` and `doctor` without falling back to `cargo run -p port-cli`. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && just package-proof x86_64-unknown-linux-gnu', proof: ac-1.log -->
- [x] [SRS-NFR-03/AC-02] The package proof remains repo-local and can be recorded without external release credentials or hosted publication infrastructure. <!-- [SRS-NFR-03/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && vhs validate artifacts/package-proof.tape && rg -n "bash scripts/package-proof.sh x86_64-unknown-linux-gnu" artifacts/package-proof.tape', proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VDchsxHfm/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VDchsxHfm/EVIDENCE/ac-2.log)

### Publish Installable Support Matrix And Release Contract
- **ID:** VDchsxHfp
- **Status:** done

#### Summary
Publish the first installable Linux and macOS support matrix and rewrite the
release checklist so the package workflow, platform boundaries, and canonical
validation path are explicit in the operator-facing docs.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `README.md`, `RELEASE.md`, and the install-focused docs publish the first supported Linux and macOS targets, their canonical package artifact, and the unsupported-environment boundary for this slice. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "Supported Targets|canonical package|WSL|remote Linux host|macOS" README.md RELEASE.md docs', proof: ac-1.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VDchsxHfp/EVIDENCE/ac-1.log)



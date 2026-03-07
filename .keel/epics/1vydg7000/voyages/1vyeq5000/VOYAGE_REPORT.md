# VOYAGE REPORT: Cloud Linux Control Lane

## Voyage Metadata
- **ID:** 1vyeq5000
- **Epic:** 1vydg7000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Publish Cloud Support Matrix
- **ID:** 1vyerX000
- **Status:** done

#### Summary
Publish the remote Linux support matrix, provider boundaries, operator workflow,
and explicit PVM drop decision in the README and supporting docs so the cloud
lane is discoverable and honest at the CLI product surface.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] README and supporting docs publish the cloud Linux support matrix and remote operator workflow using canonical Port CLI and model terms. <!-- [SRS-04/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && /tmp/port-target/debug/port --help | rg -n "Cloud Linux|AWS|GCP|Azure|PVM" && rg -n "Cloud Linux|AWS|GCP|Azure|remote Linux|port doctor|port machine launch" /home/alex/workspace/spoke-sh/port/README.md /home/alex/workspace/spoke-sh/port/docs/operators.md /home/alex/workspace/spoke-sh/port/docs/cloud.md', proof: ac-1.log-->
- [x] [SRS-05/AC-01] The shipped docs and planning artifacts record the explicit research-backed decision to drop the PVM lane from the MVP. <!-- [SRS-05/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "PVM|protected VM|confidential VM|drop" /home/alex/workspace/spoke-sh/port/README.md /home/alex/workspace/spoke-sh/port/docs/cloud.md /home/alex/workspace/spoke-sh/port/.keel/epics/1vydg7000/voyages/1vyeq5000/SRS.md /home/alex/workspace/spoke-sh/port/.keel/epics/1vydg7000/voyages/1vyeq5000/SDD.md', proof: ac-2.log-->

#### Implementation Insights
- **1vyeP0000: Anchor Platform Guidance On `port doctor`**
  - Insight: The most stable support contract is to document the intended workflow and then use `port doctor` as the runtime gate instead of promising environment capabilities that vary across hosts, especially in WSL-backed setups
  - Suggested Action: Keep README, platform docs, and CLI help centered on the exact `port doctor` boundary when platform support depends on Linux host capabilities
  - Applies To: README, `docs/operators.md`, CLI help text, diagnostics
  - Category: process


#### Verified Evidence
- [ac-1.log](../../../../stories/1vyerX000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vyerX000/EVIDENCE/ac-2.log)

### Model Cloud Linux Providers
- **ID:** 1vyerj000
- **Status:** done

#### Summary
Extend the Port host model and canonical example config so remote Linux targets
carry explicit provider identity for generic Linux, AWS, GCP, and Azure
instead of relying on implicit SSH-only intent.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] The canonical Port model distinguishes local Linux, generic remote Linux, AWS, GCP, and Azure host targets with explicit provider identity. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-model && rg -n "provider\\s*=\\s*\"(local|generic-linux|aws|gcp|azure)\"" /home/alex/workspace/spoke-sh/port/examples/port.toml', proof: ac-1.log-->

#### Implementation Insights
- **1vyezc000: Parse-Test Canonical Example Configs**
  - Insight: String-matching example config content is not enough once the example carries workflow-critical provider identity; a parse test catches drift between the checked-in example and the shared model.
  - Suggested Action: Add a `PortConfig::from_path` test for canonical example files whenever model shape changes.
  - Applies To: `examples/*.toml`, `crates/port-model/src/lib.rs`
  - Category: testing


#### Verified Evidence
- [ac-1.log](../../../../stories/1vyerj000/EVIDENCE/ac-1.log)

### Implement Remote Linux Diagnostics
- **ID:** 1vyetE000
- **Status:** done

#### Summary
Teach the canonical CLI/runtime surfaces to understand remote Linux provider
intent, report support boundaries in `port doctor`, and fail fast with
actionable guidance when operators try to launch against unimplemented remote
cloud hosts.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] `port doctor` emits provider-aware diagnostics for generic remote Linux, AWS, GCP, and Azure host targets without overstating implementation status. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime -p port-cli && /tmp/port-target/debug/port --config examples/port.toml doctor', proof: ac-1.log-->
- [x] [SRS-03/AC-01] `port machine launch` rejects remote cloud hosts with provider-specific next-step guidance instead of a generic unsupported-host error. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && set +e; output=$(/tmp/port-target/debug/port --config examples/port.toml machine launch --machine cloud-aws 2>&1); status=$?; printf "%s\n" "$output"; test "$status" -eq 1; printf "%s\n" "$output" | rg -q "AWS remains a justified future Firecracker lane"; printf "%s\n" "$output" | rg -q "Run Port on the AWS Linux host itself"', proof: ac-2.log-->

#### Implementation Insights
- **1vyf2b000: Guard Remote Launches Before Local Preflight**
  - Insight: Remote-provider launch requests should be rejected before Linux-local preflight runs, otherwise missing local prerequisites can hide the real provider-specific support boundary.
  - Suggested Action: Resolve the target machine and host first, return provider-aware guidance for remote lanes, and run `/dev/kvm` or local-binary checks only for actual local-launch paths.
  - Applies To: `crates/port-runtime/src/lib.rs`, launch guards, future remote control lanes
  - Category: architecture


#### Verified Evidence
- [ac-1.log](../../../../stories/1vyetE000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vyetE000/EVIDENCE/ac-2.log)



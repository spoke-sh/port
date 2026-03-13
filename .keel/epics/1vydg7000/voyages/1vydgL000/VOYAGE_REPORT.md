# VOYAGE REPORT: Local Linux CLI Runtime

## Voyage Metadata
- **ID:** 1vydgL000
- **Epic:** 1vydg7000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 5/5 stories complete

## Implementation Narrative
### Bootstrap Port Workspace And CLI
- **ID:** 1vydgl000
- **Status:** done

#### Summary
Create the Rust workspace, canonical `port` CLI skeleton, and shared model that
subsequent runtime, guest-agent, and artifact stories can extend without
rewriting the command surface.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] A Rust workspace exists with the canonical `port` binary and shared model/protocol crates checked in. <!-- [SRS-01/AC-01] verify: manual, proof: ac-1.log-->
- [x] [SRS-01/AC-02] `port --help` and the top-level command tree expose the planned artifact, machine, and guest surfaces with coherent help text. <!-- [SRS-01/AC-02] verify: cargo run -p port-cli -- guest --help, proof: ac-2.log-->
- [x] [SRS-01/AC-03] Model serialization and CLI parsing are covered by automated tests runnable through the repo test command. <!-- [SRS-01/AC-03] verify: cargo test, proof: ac-3.log-->

#### Implementation Insights
- **1vye3K000: Verify Annotations Are Required For Story Evidence**
  - Insight: `keel story record` ignores ACs unless each criterion includes an inline HTML comment that repeats the AC ID and declares the verification technique or command, for example ``.
  - Suggested Action: Add verify annotations while authoring or refining stories, before starting implementation, so evidence recording and submit gates do not stall later.
  - Applies To: `.keel/stories/*/README.md`
  - Category: process


#### Verified Evidence
- [ac-1.log](../../../../stories/1vydgl000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vydgl000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vydgl000/EVIDENCE/ac-3.log)

### Document Operator Workflows
- **ID:** 1vydgm000
- **Status:** done

#### Summary
Document the supported Linux, macOS, and Windows operator workflows and make
the CLI surface reflect those platform constraints instead of leaving them
implicit.

#### Acceptance Criteria
- [x] [SRS-07/AC-01] README and supporting docs explain the Linux local-launch workflow end-to-end using canonical CLI commands. <!-- [SRS-07/AC-01] verify: rg -n "Linux Local Workflow|artifacts build --artifact demo-kernel|machine launch --machine demo" /home/alex/workspace/spoke-sh/port/README.md /home/alex/workspace/spoke-sh/port/docs/operators.md, proof: ac-2.log-->
- [x] [SRS-07/AC-02] macOS operator guidance explains the supported remote-host workflow and explicitly states why local Firecracker launch is unsupported. <!-- [SRS-07/AC-02] verify: rg -n "macOS|Linux host|unsupported" /home/alex/workspace/spoke-sh/port/README.md /home/alex/workspace/spoke-sh/port/docs/operators.md, proof: ac-3.log-->
- [x] [SRS-07/AC-03] Windows operator guidance explains the supported Linux or WSL-host workflow and explicitly states current constraints. <!-- [SRS-07/AC-03] verify: rg -n "Windows|WSL|/dev/kvm|remote Linux host" /home/alex/workspace/spoke-sh/port/README.md /home/alex/workspace/spoke-sh/port/docs/operators.md, proof: ac-4.log-->
- [x] [SRS-07/AC-04] CLI help and diagnostics align with the documented platform support matrix. <!-- [SRS-07/AC-04] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && nix develop -c cargo run -p port-cli -- --help && nix develop -c cargo run -p port-cli -- doctor', proof: ac-5.log-->

#### Implementation Insights
- **1vyeP0000: Anchor Platform Guidance On `port doctor`**
  - Insight: The most stable support contract is to document the intended workflow and then use `port doctor` as the runtime gate instead of promising environment capabilities that vary across hosts, especially in WSL-backed setups
  - Suggested Action: Keep README, platform docs, and CLI help centered on the exact `port doctor` boundary when platform support depends on Linux host capabilities
  - Applies To: README, `docs/operators.md`, CLI help text, diagnostics
  - Category: process


#### Verified Evidence
- [ac-1.log](../../../../stories/1vydgm000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vydgm000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vydgm000/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/1vydgm000/EVIDENCE/ac-4.log)
- [ac-5.log](../../../../stories/1vydgm000/EVIDENCE/ac-5.log)

### Implement Local Firecracker Launch
- **ID:** 1vydim000
- **Status:** done

#### Summary
Implement the Linux host preflight and the first real local Firecracker launch
path, including runtime state directories, Firecracker config generation, and a
recorded end-to-end proof.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] `port doctor` validates Linux host support, `/dev/kvm`, Firecracker availability, and required tooling with actionable errors. <!-- [SRS-02/AC-01] verify: cargo run -p port-cli -- --config /tmp/port-proof/port.toml doctor, proof: ac-2.log-->
- [x] [SRS-02/AC-02] Failure modes for unsupported hosts and launch failures preserve actionable diagnostics. <!-- [SRS-02/AC-02] verify: bash /tmp/port-proof/verify-launch-failure.sh, proof: ac-3.log-->
- [x] [SRS-03/AC-01] `port machine launch` boots a Firecracker VM from Port-managed artifacts and records runtime metadata and log locations. <!-- [SRS-03/AC-01] verify: bash /tmp/port-proof/verify-launch-success.sh, proof: ac-5.log-->
- [x] [SRS-03/AC-02] Automated tests cover Firecracker config generation and runtime path/state behavior without requiring KVM in every test. <!-- [SRS-03/AC-02] verify: cargo test, proof: ac-6.log-->

#### Implementation Insights
- **1vye8L000: Firecracker 1.14 Uses `smt` In `machine-config`**
  - Insight: Current Firecracker 1.14 rejects the older `machine-config.ht_enabled` field and expects `machine-config.smt` instead. Using the older field fails fast during JSON parsing before the microVM starts.
  - Suggested Action: Match generated config fields to the live Firecracker binary in the dev shell and confirm with an executable launch proof before trusting older examples.
  - Applies To: `crates/port-runtime/*`, Firecracker config generation, local launch proofs
  - Category: architecture


#### Verified Evidence
- [ac-1.log](../../../../stories/1vydim000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vydim000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vydim000/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/1vydim000/EVIDENCE/ac-4.log)
- [ac-5.log](../../../../stories/1vydim000/EVIDENCE/ac-5.log)
- [ac-6.log](../../../../stories/1vydim000/EVIDENCE/ac-6.log)

### Deliver Guest Agent Capabilities
- **ID:** 1vydip000
- **Status:** done

#### Summary
Implement the guest agent transport and expose `exec`, `copy`, `pty`, `logs`,
and `forward` through the Port CLI and shared protocol.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] The guest agent protocol supports request/response flows for `exec`, `copy`, `pty`, `logs`, and `forward`. <!-- [SRS-04/AC-01] verify: cargo test -p port-agent-protocol -p port-guest-agent, proof: ac-1.log-->
- [x] [SRS-04/AC-02] The canonical CLI exposes `port guest exec`, `port guest copy`, `port guest pty`, `port guest logs`, and `port guest forward`. <!-- [SRS-04/AC-02] verify: bash -lc 'cargo run -p port-cli -- guest --help && cargo test -p port-cli --test guest_commands', proof: ac-2.log-->
- [x] [SRS-04/AC-03] Automated tests cover protocol framing and at least one happy-path behavior for each guest capability. <!-- [SRS-04/AC-03] verify: cargo test, proof: ac-3.log-->

#### Implementation Insights
- **1vyeM0000: Prefer In-Process Daemons For Workspace CLI Integration Tests**
  - Insight: Spawning the daemon crate in-process through a dev-dependency is more reliable than discovering a sibling workspace binary from the test harness
  - Suggested Action: Prefer `thread::spawn` plus the daemon library entrypoint for workspace-local CLI integration tests unless the binary packaging itself is under test
  - Applies To: `crates/*/tests/*.rs`, workspace daemons, CLI integration tests
  - Category: testing


#### Verified Evidence
- [ac-1.log](../../../../stories/1vydip000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vydip000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vydip000/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/1vydip000/EVIDENCE/ac-4.log)

### Build Artifact Pipelines And Docs
- **ID:** 1vydit000
- **Status:** done

#### Summary
Build the kernel and guest-image pipelines used by the local MVP path, validate
their outputs, and document the artifact contracts and operator-facing build
workflow.

#### Acceptance Criteria
- [x] [SRS-05/AC-01] A reproducible kernel build pipeline exists in-repo and emits a documented kernel artifact for Firecracker. <!-- [SRS-05/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && nix develop -c cargo run -p port-cli -- --config /home/alex/workspace/spoke-sh/port/examples/port.toml artifacts build --artifact demo-kernel && rg -n "demo-kernel|Kernel Artifact" /home/alex/workspace/spoke-sh/port/README.md /home/alex/workspace/spoke-sh/port/docs/artifacts.md', proof: ac-2.log-->
- [x] [SRS-05/AC-02] Validation commands or checks exist for kernel and guest-image artifacts and are recorded as evidence. <!-- [SRS-05/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && nix develop -c cargo run -p port-cli -- --config /home/alex/workspace/spoke-sh/port/examples/port.toml artifacts validate --artifact demo-kernel && nix develop -c cargo run -p port-cli -- --config /home/alex/workspace/spoke-sh/port/examples/port.toml artifacts validate --artifact demo-guest', proof: ac-3.log-->
- [x] [SRS-06/AC-01] A reproducible guest-image build pipeline exists in-repo and emits a documented guest-image artifact with the Port guest agent. <!-- [SRS-06/AC-01] verify: bash /tmp/port-proof-artifacts/verify-built-guest-launch.sh, proof: ac-5.log-->

#### Implementation Insights
- **1vyeN0000: Build Firecracker Rootfs Images Without Privileged Mounts**
  - Insight: `mkfs.ext4 -d` plus `ldd`-discovered shared libraries is enough to assemble a bootable ext4 guest image carrying dynamic binaries like BusyBox and `port-guest-agent`
  - Suggested Action: Prefer staging-directory image assembly with `mkfs.ext4 -d`, `e2fsck`, and `debugfs` before introducing mount-based image mutation tooling
  - Applies To: `scripts/artifacts/*.sh`, guest image pipelines, future cloud image assembly
  - Category: architecture


#### Verified Evidence
- [ac-1.log](../../../../stories/1vydit000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vydit000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vydit000/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/1vydit000/EVIDENCE/ac-4.log)
- [ac-5.log](../../../../stories/1vydit000/EVIDENCE/ac-5.log)



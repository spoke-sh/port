# VOYAGE REPORT: Hosted Control And Substrate Foundations

## Voyage Metadata
- **ID:** 1vz2ky000
- **Epic:** 1vz2eV000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 4/4 stories complete

## Implementation Narrative
### Model Substrates And Protection Modes
- **ID:** 1vz2oQ000
- **Status:** done

#### Summary
Extend Port's canonical model, validation, and operator docs so runtime
capability is expressed through substrate, protection mode, architecture, and
artifact compatibility instead of through provider identity alone.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `port-model` can represent backend, protection mode, architecture, and artifact-compatibility metadata for machines and artifacts while the existing local Firecracker/KVM sample config still parses and validates. <!-- [SRS-01/AC-01] verify: env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-model, proof: ac-1.log-->
- [x] [SRS-01/AC-02] Port publishes canonical substrate terms and a support matrix covering Firecracker/KVM, Firecracker/PVM, Cloud Hypervisor, Apple Virtualization Framework, and the explicit arm64 protected-virtualization research lane. <!-- [SRS-01/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "Execution Lanes|Cloud Hypervisor|Apple Virtualization Framework|research lane|protection_mode" README.md docs/cloud.md docs/operators.md examples/port.toml', proof: ac-2.log-->
- [x] [SRS-01/AC-03] Unsupported backend, protection-mode, or architecture combinations fail fast with actionable model or CLI diagnostics instead of silently degrading. <!-- [SRS-01/AC-03] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime doctor_report_includes_machine_lane_checks -- --exact && env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime launch_rejects_unsupported_pvm_artifact_contract -- --exact', proof: ac-3.log-->

#### Implementation Insights
- **1vz3A9000: Separate Architecture From Protection-Mode Support**
  - Insight: General architecture support and protected-virtualization support cannot share one boolean or one token. Port needs explicit substrate, protection-mode, and architecture fields so it can say “arm64 exists” without implying “arm64 PVM is shipped.”
  - Suggested Action: Keep machine compatibility validation and docs keyed on substrate plus protection mode plus resolved architecture, and reject unsupported combinations explicitly.
  - Applies To: crates/port-model/**, crates/port-runtime/**, docs/**, examples/port.toml
  - Category: architecture


#### Verified Evidence
- [ac-1.log](../../../../stories/1vz2oQ000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vz2oQ000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vz2oQ000/EVIDENCE/ac-3.log)

### Add Machine Inventory Status And Stop
- **ID:** 1vz2oY000
- **Status:** done

#### Summary
Add the first real machine lifecycle surfaces Port is missing: local inventory,
status, and stop commands backed by runtime manifests, pid inspection, and
coherent CLI output.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] `port machine list` enumerates machines under the selected runtime root and reports their lifecycle state from manifests plus live process inspection. <!-- [SRS-02/AC-01] verify: env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime list_machines_reports_running_stale_and_malformed_runtime_entries && env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-cli help_includes_primary_surfaces, proof: ac-1.log-->
- [x] [SRS-02/AC-02] `port machine status --machine <name>` prints actionable runtime metadata including liveness, pid, runtime paths, and troubleshooting log references. <!-- [SRS-02/AC-02] verify: env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime machine_status_reports_runtime_paths_for_known_machine && env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-cli parses_machine_lifecycle_arguments, proof: ac-2.log-->
- [x] [SRS-02/AC-03] `port machine stop --machine <name>` safely stops a Port-owned local machine and reports the resulting lifecycle outcome through the canonical CLI. <!-- [SRS-02/AC-03] verify: env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime stop_machine_terminates_live_port_owned_process && env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-cli parses_machine_lifecycle_arguments, proof: ac-3.log-->
- [x] [SRS-03/AC-04] Missing, stale, or malformed runtime state produces explicit diagnostics instead of silent skips or ambiguous failures. <!-- [SRS-03/AC-04] verify: env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime machine_status_reports_missing_and_malformed_runtime_state, proof: ac-4.log-->

#### Implementation Insights
- **1vz3Mb000: Runtime Lifecycle Should Key Off Runtime State, Not Config**
  - Insight: `launch` is model-backed, but `list`, `status`, and `stop` should key off runtime manifests and PID inspection instead of reloading the machine model. That keeps lifecycle commands usable after a VM already exists and matches the control-plane direction for hosted Port.
  - Suggested Action: Treat runtime-root inspection data as the source of truth for post-launch lifecycle commands, and only require the model for launch-time validation or artifact resolution.
  - Applies To: crates/port-runtime/**, crates/port-cli/**, docs/operators.md, README.md
  - Category: architecture


#### Verified Evidence
- [ac-1.log](../../../../stories/1vz2oY000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vz2oY000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vz2oY000/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/1vz2oY000/EVIDENCE/ac-4.log)

### Publish Hosted Node Agent Contract
- **ID:** 1vz2oh000
- **Status:** done

#### Summary
Define the first canonical hosted-Port contract: a node-local agent plus central
control plane that preserve today's guest-operation model while adding remote
lifecycle ownership, transport brokering, and inventory.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] Port publishes a canonical hosted-control document describing node-agent responsibilities, control-plane responsibilities, and machine lifecycle ownership for local versus hosted execution. <!-- [SRS-04/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "Node Agent|Control Plane|Lifecycle Ownership|Local Port today|Hosted Port planned" docs/hosted.md README.md', proof: ac-1.log-->
- [x] [SRS-04/AC-02] The contract explains how guest `exec`, `copy`, `pty`, `logs`, and `forward` are brokered through the hosted product without replacing the current guest protocol semantics. <!-- [SRS-04/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "exec|copy|pty|logs|forward|guest protocol|broker|connect_guest" docs/hosted.md', proof: ac-2.log-->
- [x] [SRS-04/AC-03] README and linked docs surface the hosted contract and the current support matrix so operators can distinguish shipped local behavior from planned hosted behavior. <!-- [SRS-04/AC-03] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && env CARGO_TARGET_DIR=/tmp/port-target cargo run -p port-cli -- --help >/tmp/1vz2oh000-help.verify && rg -n "Hosted Control Preview|docs/hosted.md|Hosted Control:" README.md docs/cloud.md docs/operators.md crates/port-cli/src/lib.rs', proof: ac-3.log-->

#### Implementation Insights
- **1vz3N8000: Hosted Port Should Broker The Guest Protocol, Not Replace It**
  - Insight: Port already has a usable guest protocol and CLI vocabulary. The hosted layer should add routing, ownership, and auth around that protocol instead of inventing a second hosted-only guest API.
  - Suggested Action: Keep hosted design and implementation work centered on tunneling the existing guest protocol through node agents and control-plane sessions, with the CLI remaining the same surface in local and hosted modes.
  - Applies To: docs/hosted.md, crates/port-runtime/**, crates/port-cli/**, future hosted-control crates
  - Category: architecture


#### Verified Evidence
- [ac-1.log](../../../../stories/1vz2oh000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vz2oh000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vz2oh000/EVIDENCE/ac-3.log)

### Define Artifact Mobility Commands And Contracts
- **ID:** 1vz2on000
- **Status:** done

#### Summary
Turn artifacts into a real product surface for local and remote use by defining
canonical references, compatibility metadata, and discoverable build, push, and
pull semantics.

#### Acceptance Criteria
- [x] [SRS-05/AC-01] Port defines canonical artifact-reference and compatibility concepts covering local outputs, remote references, architecture, backend, and protection-mode variants. <!-- [SRS-05/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && env CARGO_TARGET_DIR=/tmp/port-target cargo test -q -p port-model', proof: ac-1.log-->
- [x] [SRS-06/AC-01] The CLI surface and help text expose discoverable artifact mobility commands or reserved subcommands for build, push, and pull workflows. <!-- [SRS-06/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo run -q -p port-cli -- artifacts --help && cargo run -q -p port-cli -- artifacts push --help', proof: ac-2.log-->
- [x] [SRS-06/AC-02] Port publishes operator-facing documentation for local build, remote pull, and compatibility-selection flows using the new artifact vocabulary. <!-- [SRS-06/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "artifacts push|artifacts pull|Artifact Contracts|file-backed store|--architecture|reference|variants|cache" README.md docs/operators.md docs/artifacts.md docs/cloud.md', proof: ac-3.log-->
- [x] [SRS-05/AC-04] The story defines concrete verification hooks for artifact mobility behavior through tests, docs review, and CLI-level evidence. <!-- [SRS-05/AC-04] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && env CARGO_TARGET_DIR=/tmp/port-target cargo test -q -p port-cli && rm -rf artifact-store/demo-fs .port/cache artifacts/kernel/demo && mkdir -p artifacts/kernel/demo/x86_64/firecracker/standard && printf demo-kernel-proof > artifacts/kernel/demo/x86_64/firecracker/standard/vmlinux && cargo run -q -p port-cli -- --config examples/port.toml artifacts push --artifact demo-kernel --architecture x86-64 && rm -f artifacts/kernel/demo/x86_64/firecracker/standard/vmlinux && cargo run -q -p port-cli -- --config examples/port.toml artifacts pull --artifact demo-kernel --architecture x86-64 && rm -rf artifact-store/demo-fs .port/cache artifacts/kernel/demo', proof: ac-4.log-->

#### Implementation Insights
- **1vz3rU000: Noninteractive Story Record Needs Editor Override**
  - Insight: `keel story record` still opens a manual-evidence editor even for command proofs unless the editor exits immediately; setting `EDITOR=true` keeps the command proof path noninteractive.
  - Suggested Action: Use `EDITOR=true nix develop -c keel story record ... --cmd "<command>"` for automated proof capture and only fall back to a PTY editor when a manual note is genuinely needed.
  - Applies To: `.keel/stories/*`, proof recording workflow
  - Category: process


#### Verified Evidence
- [ac-1.log](../../../../stories/1vz2on000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vz2on000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vz2on000/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/1vz2on000/EVIDENCE/ac-4.log)



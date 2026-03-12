# Attached Volume Contract Foundations - SRS

> Define the canonical volume contract and the first direct-runtime
> attached-volume slice with proof-backed operator workflow.

**Epic:** [VDcStQqlo](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Add an explicit attached-volume contract to the shared machine
  model that is distinct from boot artifacts and rootfs settings.
- [SCOPE-02] Support one persistent `host-file` data volume for the local
  standard Firecracker lifecycle through canonical `machine launch`, `status`,
  and `stop` surfaces.
- [SCOPE-03] Surface direct-runtime storage prerequisites, backend detail, and
  ownership guidance through docs, validation, and lifecycle output.
- [SCOPE-04] Publish one proof-backed operator workflow for the first attached
  volume lane.

### Out of Scope

- [SCOPE-90] Hosted control-plane and SSH-managed attached-volume routing.
- [SCOPE-91] Ephemeral managed volumes, snapshots, clone or resize semantics,
  and multi-volume attachment.
- [SCOPE-92] Cloud-provider native disks, storage scheduling, CSI, or service
  orchestration.
- [SCOPE-93] Cloud Hypervisor, AVF, PVM, or guest-side filesystem and mount
  automation.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Firecracker's existing multi-drive config surface is sufficient for the first attached-volume slice. | dependency | The voyage would need a deeper launcher redesign before any storage work can ship. |
| One persistent file-backed data volume is enough to prove the first storage contract. | assumption | The voyage would need a larger storage-class or lifecycle design up front. |
| The shared machine model can grow an attached-volume section without disturbing the current artifact and rootfs workflow. | dependency | The voyage would risk broad compatibility work instead of a bounded storage slice. |
| Human-reviewable proof should continue to use the repo's recording-capable proof system. | dependency | The voyage would need a different proof strategy and mission-review surface. |

## Constraints

- Preserve one canonical `port machine ...` command family; do not introduce a
  second storage-specific CLI.
- Keep `kernel`, `guest_image`, and `rootfs_read_only` as boot concerns and
  treat attached volumes as a separate operator-visible contract.
- Use one explicit backend token for the first slice: persistent `host-file`.
- Fail fast when a machine requests an attached volume the local Firecracker
  lane cannot satisfy.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Port must add an attached-volume definition to the shared machine model and validate one persistent `host-file` contract separately from the current `guest_image` rootfs artifact. | SCOPE-01 | FR-01 | automated test + config proof |
| SRS-02 | `port doctor`, validation, and operator docs must distinguish attached-volume readiness from rootfs artifact readiness and explain the first-slice lane contract. | SCOPE-03 | FR-03 | automated test + command proof |
| SRS-03 | The local standard Firecracker launch path must attach one additional data drive for machines that declare a supported attached volume while preserving current rootfs-only behavior for machines that do not. | SCOPE-02 | FR-02 | automated test + command proof |
| SRS-04 | CLI-visible success and failure surfaces for attached-volume machines must keep backend, host path, machine, route, and ownership detail explicit, including fail-fast guidance for hosted and SSH-owned requests that remain unsupported in this slice. | SCOPE-02, SCOPE-03 | FR-03 | automated test + command proof |
| SRS-05 | The voyage must publish direct-runtime storage guidance and a proof-backed operator workflow for the first attached-volume lane. | SCOPE-03, SCOPE-04 | FR-04 | inspection + recording |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Unsupported attached-volume requests must fail fast with explicit machine, host, lane, backend, and path detail. | SCOPE-02, SCOPE-03 | NFR-01 | automated test + command proof |
| SRS-NFR-02 | Machines without attached volumes must preserve the existing artifact and rootfs lifecycle behavior without silent changes. | SCOPE-01, SCOPE-02 | NFR-02 | automated regression test |
| SRS-NFR-03 | Verification for this voyage must use repo-local techniques recommended by Keel for this repository: Rust tests, command proofs, and a recording-backed human proof path. | SCOPE-02, SCOPE-03, SCOPE-04 | NFR-03 | board review + command proof + recording |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Story Coverage Plan

| Story | Coverage |
|-------|----------|
| [VDfF1dZM9](../../../../stories/VDfF1dZM9/README.md) Introduce Canonical Volume And Attachment Model | SRS-01, SRS-NFR-02 |
| [VDfF1csOC](../../../../stories/VDfF1csOC/README.md) Implement Local Attached Volume Launch Path | SRS-02, SRS-03, SRS-NFR-02 |
| [VDfF1cZOD](../../../../stories/VDfF1cZOD/README.md) Add Attached Volume Lane Guidance | SRS-03, SRS-NFR-01 |
| [VDfF1dVOF](../../../../stories/VDfF1dVOF/README.md) Publish Attached Volume Operator Proof | SRS-04, SRS-NFR-03 |

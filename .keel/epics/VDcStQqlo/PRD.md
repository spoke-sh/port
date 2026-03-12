# Cloud Block Storage Normalization - Product Requirements

> Port needs one canonical storage contract that separates boot artifacts from
> operator-visible attached volumes so local, hosted, and SSH-owned Firecracker
> lanes can support stateful workloads without pretending Port already ships a
> full storage platform.

## Problem Statement

Port already has a strong artifact and guest-image story, but it does not yet
publish a first-class portable block-storage contract for persistent or
ephemeral attached volumes. The current rootfs- and artifact-centric model is
credible for booting machines, but it is not enough for operator-facing
stateful workloads or for the cloud storage normalization the product horizon
now calls for.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Define one canonical storage vocabulary that separates boot artifacts from attached data volumes across local, hosted, and SSH-managed Firecracker lanes. | Planning artifacts and shared model changes keep `kernel` and `guest_image` as boot artifacts while introducing explicit attached-volume semantics. | Epic PRD, voyage SRS/SDD, and downstream stories preserve one operator model for rootfs plus attached volumes. |
| GOAL-02 | Plan and ship a first executable attached-volume slice without expanding into CSI, distributed storage, or service orchestration. | One voyage scopes a bounded attached-volume lane with executable stories and proof-backed verification. | A Firecracker-first attached-volume slice is planned and ready for operator delivery. |
| GOAL-03 | Make storage ownership, backend detail, and persistence semantics explicit before launch and in lifecycle surfaces. | Docs, validation, and lifecycle output tell operators which host owns a volume, which backend is in use, and whether the volume is ephemeral or persistent. | Operators can reason about storage behavior without reading runtime code. |
| GOAL-04 | Preserve the existing artifact, rootfs, and machine lifecycle story for machines that do not opt into attached volumes. | Existing artifact commands and current machine launch flows continue to work without compatibility shims or silent behavior changes. | No planned slice regresses the shipped boot-artifact workflow. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Solo Operator | Runs Port directly on one Linux host or over SSH and wants a small, explicit way to attach writable storage to a VM. | One canonical attached-volume workflow that keeps `port machine ...` as the control surface. |
| Platform Engineer | Maintains hosted or remote Firecracker nodes and needs storage ownership to stay explicit at the host boundary. | Clear backend, placement, and persistence semantics for attached volumes. |
| Stateful Workload Evaluator | Wants to judge whether Port can eventually support databases, k3s, and other stateful services. | A credible first storage contract that higher-level workload work can build on. |

## Scope

### In Scope

- [SCOPE-01] A shared machine-storage contract that distinguishes boot
  artifacts (`kernel`, `guest_image`, `rootfs_read_only`) from attached data
  volumes.
- [SCOPE-02] One bounded attached-volume lane for standard Firecracker
  execution across local runtime, hosted control-plane ownership, and
  SSH-managed remote Linux hosts.
- [SCOPE-03] Explicit backend, ownership, and persistence language in config,
  lifecycle output, docs, and planning artifacts.
- [SCOPE-04] Proof-backed operator guidance for the first attached-volume
  workflow.

### Out of Scope

- [SCOPE-90] CSI, distributed storage, replication, snapshots, resize, or
  generic storage-service orchestration.
- [SCOPE-91] Cloud-provider native disk APIs, managed volume provisioning, or
  scheduler-driven storage placement beyond the first bounded lane.
- [SCOPE-92] Substrate parity for Cloud Hypervisor, AVF, or Firecracker PVM.
- [SCOPE-93] Guest-side filesystem automation, service templates, or
  Kubernetes persistence orchestration.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must extend the shared machine model with explicit attached-volume definitions that are distinct from boot artifacts and guest rootfs configuration. | GOAL-01, GOAL-04 | must | Storage normalization starts by separating rootfs artifact selection from operator-visible data-volume semantics. |
| FR-02 | Port must support one bounded attached-volume workflow through canonical `machine launch`, `status`, and `stop` surfaces for the current Firecracker standard lane. | GOAL-02, GOAL-03 | must | The first storage slice needs an executable workflow, not only new vocabulary. |
| FR-03 | Port must keep storage backend, persistence mode, host ownership, and route detail explicit across local, hosted, and SSH-managed execution. | GOAL-01, GOAL-03 | must | Storage work becomes misleading if ownership or backend detail is hidden behind generic cloud language. |
| FR-04 | Port must publish a proof-backed operator workflow for the first attached-volume lane, including at least one human-reviewable artifact path. | GOAL-02, GOAL-03 | should | The first storage lane should be reviewable by humans through the same mission and proof surfaces as other product slices. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Unsupported storage backends, lanes, or host prerequisites must fail fast with explicit machine, host, and volume detail. | GOAL-03, GOAL-04 | must | Operators need actionable errors instead of implicit fallback to rootfs-only behavior. |
| NFR-02 | Machines that do not opt into attached volumes must preserve the current artifact and rootfs lifecycle behavior without silent changes. | GOAL-04 | must | Storage work cannot regress the credible local, hosted, and SSH machine lanes already shipped. |
| NFR-03 | Verification for the first storage slice must use recommended repo-local techniques, including Rust tests, command proofs, and at least one recording-backed human proof path. | GOAL-02, GOAL-03 | must | The board should capture executable evidence, not only planning prose. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Prove shared-model and runtime behavior through story-level Rust tests mapped
  to voyage requirements.
- Use command proofs and a recording-backed human proof path to demonstrate the
  first attached-volume workflow end to end.
- Validate non-functional posture with explicit failure-path checks for
  unsupported backends, missing paths, and unchanged rootfs-only behavior.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| One attached data volume is enough for the first storage slice. | The voyage would need a broader multi-volume or storage-class design before any delivery can start. | Validate in voyage SRS and story decomposition. |
| The first backend can stay explicitly file-backed on the execution host while still representing a credible cross-lane storage contract. | The epic would need to jump directly into provider-native volume APIs or host-group storage capabilities. | Validate against the current Firecracker launch seam and hosted/SSH ownership model. |
| Existing Firecracker lifecycle surfaces can carry storage ownership detail without inventing a second CLI or service layer. | The epic would require broader CLI or control-plane refactoring. | Validate in voyage design and the first operator proof slice. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should the first storage contract live entirely inside `MachineSpec`, or should it introduce a reusable top-level volume catalog immediately? | Epic owner | Open |
| How should persistent file-backed volumes interact with hosted node placement and SSH-owned runtime roots? | Epic owner | Open |
| Firecracker already supports multiple drives, but the current runtime metadata and status surfaces are still rootfs-centric. | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] The epic clearly separates boot artifacts and rootfs behavior from
      operator-visible attached-volume semantics.
- [ ] At least one voyage plans a bounded attached-volume workflow with
      executable stories and traceable verification.
- [ ] The first storage slice stays explicit about backend and persistence
      detail instead of claiming generic cloud-volume support.
- [ ] Follow-on work for k3s and stateful workloads can point at this storage
      contract rather than inventing ad hoc disk semantics.
<!-- END SUCCESS_CRITERIA -->

## Research Analysis

*From bearing assessment:*

### Findings

- Current Port storage is artifact-first and rootfs-first rather than
  volume-first [SRC-01][SRC-02].
- External VM-platform precedent keeps storage modes explicit and operator
  visible [SRC-03][SRC-04].
- This work would create a cleaner foundation for later stateful workloads and
  k3s stories [SRC-01][SRC-03].

### Opportunity Cost

Pursuing storage normalization before the hybrid remote story settles could
force rework in hosted placement and remote-node ownership. Even so, the topic
needs to be recorded now because stateful workloads and cloud usage both depend
on it [SRC-02][SRC-03].

### Dependencies

- Current artifact and rootfs contract [SRC-01][SRC-02]
- Explicit backend lessons from storage-oriented VM platform docs [SRC-03][SRC-04]
- Hosted placement and future stateful workload planning [SRC-02]

### Alternatives Considered

- Continue treating guest images as the only storage surface. Rejected because
  that approach does not answer the user's request for normalized block storage
  or support stateful cloud workloads well [SRC-01][SRC-02].
- Jump directly to a full storage service or CSI-style design. Rejected because
  the repo first needs a smaller shared contract for machines and hosted lanes
  [SRC-03][SRC-04].

---

*This PRD was seeded from bearing `VDcStQqlo`. See `bearings/VDcStQqlo/` for original research.*

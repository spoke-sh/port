# Ship Bootable Local Cluster Guest And Artifact Lane - Product Requirements

## Problem Statement

The previously verified cluster mission closed on operator surface and proof
criteria, but the shipped local single-node cluster lane is still not live.
`port --config examples/port.toml cluster up --cluster demo --runtime-root
<tmp> --format json` currently fails on Linux because Firecracker exits during
boot and the guest console ends in an ext4 checksum error followed by `Requested
init /init failed (error -74)`. Separately, `port --config examples/port.toml
artifacts validate --artifact demo-guest --architecture x86-64` still fails
under the installed CLI contract because the validation path resolves
`validate-guest-image.sh` under `/build/...` instead of a shipped runtime-safe
location.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Boot the shipped single-node local cluster example successfully on Linux. | `port ... cluster up --cluster demo --runtime-root <tmp> --format json` exits successfully and returns live launch output. | The checked-in example works on a Linux development host without patching downstream repos. |
| GOAL-02 | Hand off a healthy cluster directly through Port surfaces. | `cluster status --format json` reports readiness=`ready`, machine_state=`running`, kubeconfig_available=`true`, and `cluster kubeconfig --format json` works with `kubectl get nodes -o wide`. | Downstream tooling can consume the returned kubeconfig without manual rewrite. |
| GOAL-03 | Make the shipped guest artifact lane install-safe. | `artifacts validate --artifact demo-guest --architecture x86-64` succeeds from the packaged or installed Port contract. | The validate path no longer depends on build-time-only `/build/...` script locations. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Downstream Infra Operator | The upstream `spoke infra` consumer that wants Port to hand off a healthy cluster plus kubeconfig and then keep infra thin. | A live single-node local cluster handoff that works without downstream guest choreography. |
| Port Maintainer | The team shipping Port’s local cluster and artifact lanes. | A runtime-correct local cluster contract and packaged artifact workflow that match the published operator surface. |

## Scope

### In Scope

- [SCOPE-01] Booting and stabilizing the checked-in single-node local cluster lane behind `port cluster up|status|kubeconfig`.
- [SCOPE-02] Fixing the guest artifact validate path so the shipped CLI can resolve and execute its validation contract correctly.
- [SCOPE-03] Producing live verification that downstream repos can consume the local cluster handoff without rewriting kubeconfig or running extra bootstrap choreography.

### Out of Scope

- [SCOPE-04] AWS, hosted cluster, or multi-node cluster expansion.
- [SCOPE-05] Recorder or proof UX changes such as `atxt` migration.
- [SCOPE-06] Shifting cluster bootstrap ownership back into downstream `guest exec` flows or manual join-token handling.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The shipped `examples/port.toml` local cluster workflow must boot a real single-node cluster through `port cluster up` on Linux. | GOAL-01 | must | The published operator surface is not credible until the checked-in example boots live. |
| FR-02 | `port cluster status` and `port cluster kubeconfig` must hand off a live healthy cluster without downstream kubeconfig rewriting or extra guest exec choreography. | GOAL-02 | must | Downstream repos are explicitly consuming Port at this higher-level seam. |
| FR-03 | The shipped guest artifact validate path must resolve validation scripts and dependencies from install-safe locations rather than source-build-only paths. | GOAL-03 | must | The installed CLI contract has to match the artifact workflow Port publishes. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Keep scope bounded to the single-node local cluster lane and preserve explicit rejection or deferral of AWS and multi-node expansion. | GOAL-01, GOAL-02 | must | Prevents this runtime-correctness mission from expanding into a broader platform roadmap. |
| NFR-02 | Fix correctness and installability rather than compensating with extra docs, recorder work, or downstream glue. | GOAL-01, GOAL-03 | must | The problem is operational health, not missing explanation. |
| NFR-03 | Maintain Port ownership of bootstrap, health, and kubeconfig handoff once the lane is healthy. | GOAL-02 | must | Regressing to raw guest choreography would invalidate the cluster-first contract. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Local cluster boot | Live CLI run on Linux plus runtime logs | `cluster up --format json`, runtime state, and guest console evidence |
| Cluster handoff | Live CLI run plus downstream `kubectl get nodes -o wide` using the returned kubeconfig | Host-side proof that Port hands off a usable cluster directly |
| Packaged artifact validate | Installed or packaged CLI invocation of `artifacts validate` | Proof that validation no longer depends on `/build/...` script paths |
| Downstream consumption | Cross-repo manual verification in `spoke infra` | Evidence that infra reaches a “cluster handoff ready” state after Port bootstrap |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The current guest rootfs or bootstrap lane can be repaired without redesigning the entire local cluster surface. | The mission may need a larger guest or artifact refactor than currently planned. | Validate by reproducing the boot panic and isolating whether the fix belongs in guest artifacts, runtime wiring, or bootstrap inputs. |
| Downstream infra only needs a healthy cluster plus kubeconfig to proceed. | The mission may need to absorb downstream bootstrap responsibilities too early. | Reconfirm against the `spoke infra` handoff path during execution. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| What specifically corrupts or mismatches the shipped local guest image such that `/init` fails with ext4 checksum errors during boot? | Epic owner | Open |
| Should packaged artifact validation ship the scripts themselves, rewrite contract paths, or replace script-based validation for installed use? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `port --config examples/port.toml cluster up --cluster demo --runtime-root <tmp> --format json` succeeds on Linux.
- [ ] `port --config examples/port.toml cluster status --cluster demo --runtime-root <tmp> --format json` reports readiness=`ready`, machine_state=`running`, and kubeconfig_available=`true`.
- [ ] `port --config examples/port.toml cluster kubeconfig --cluster demo --runtime-root <tmp> --format json` plus `kubectl get nodes -o wide` works without rewriting the kubeconfig.
- [ ] `port --config examples/port.toml artifacts validate --artifact demo-guest --architecture x86-64` succeeds from the shipped install contract.
- [ ] The mission stays bounded to the single-node local lane and does not absorb AWS or multi-node expansion.
<!-- END SUCCESS_CRITERIA -->

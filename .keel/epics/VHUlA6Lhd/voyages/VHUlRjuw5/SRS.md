# Hosted Guest Recovery Fidelity - SRS

## Summary

Epic: VHUlA6Lhd
Goal: Make hosted guest recovery safe for production by restarting unhealthy K3s services, removing false guest-wedge inference, and surfacing trustworthy wedge evidence for downstream automation.

## Scope

### In Scope

- [SCOPE-01] Enable unhealthy restarts for hosted K3s managed services so guest runtime failures can self-heal.
- [SCOPE-02] Tighten hosted wedge classification so guest wedges require real guest heartbeat age or explicit managed-runtime failure evidence.
- [SCOPE-03] Raise machine wedge endpoint fidelity with runtime evidence fields that explain the current wedge classification and recovery posture.
- [SCOPE-04] Add focused regression coverage for the hosted recovery policy, wedge detector, recovery-action gating, and endpoint response shape.

### Out of Scope

- [SCOPE-05] Tier-3 provider actions or downstream consumer implementation in `infra`.
- [SCOPE-06] Stateful workload migration, PV mobility, or scheduler rebalancing after a worker failure.
- [SCOPE-07] Broad changes to non-hosted machine runtime policies.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Hosted K3s managed services must set `restart_on_unhealthy = true` so the guest agent restarts `k3s-agent`/`k3s-server` when their healthchecks fail while the service is still nominally running. | SCOPE-01 | FR-01 | automated |
| SRS-02 | Hosted wedge detection must not derive guest refresh age from machine placement age; a machine with no guest refresh metadata cannot classify as a guest wedge unless managed-runtime evidence independently proves a guest failure. | SCOPE-02 | FR-02 | automated |
| SRS-03 | Recovery action selection must treat an `InProgress` recovery inside the configured settle window as non-actionable so Port does not stack repeated restart/recreate actions before observing the previous attempt. | SCOPE-02, SCOPE-04 | FR-04 | automated |
| SRS-04 | The `/v1/machines/<name>/wedge` endpoint must return explicit hosted runtime evidence, including guest refresh age when present and hosted K3s service state/health details when available. | SCOPE-03 | FR-03 | automated |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The new wedge evidence fields must be additive and machine-readable so current consumers can ignore them safely while future automation can adopt them directly. | SCOPE-03 | NFR-01 | automated |
| SRS-NFR-02 | Hosted recovery behavior changes must be covered by focused tests in `port-runtime` rather than relying on manual cluster reproduction. | SCOPE-01, SCOPE-02, SCOPE-04 | NFR-02 | automated |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

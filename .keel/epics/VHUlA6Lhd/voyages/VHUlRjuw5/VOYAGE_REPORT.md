# VOYAGE REPORT: Hosted Guest Recovery Fidelity

## Voyage Metadata
- **ID:** VHUlRjuw5
- **Epic:** VHUlA6Lhd
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 1/1 stories complete

## Implementation Narrative
### Fix Hosted K3s Unhealthy Restart And Wedge Fidelity
- **ID:** VHUlRkBvx
- **Status:** done

#### Summary
Harden hosted recovery after the worker-2 production failure by making hosted K3s services restart on unhealthy healthchecks, removing false guest-wedge inference from machine placement age, and exporting trustworthy runtime evidence on the machine wedge endpoint.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Hosted K3s service policy restarts unhealthy hosted agent/server services, with automated coverage for the policy contract. <!-- [SRS-01/AC-01] verify: cargo test -p port-runtime hosted_k3s_server_healthcheck_requires_runtime_and_readyz -- --nocapture && cargo test -p port-runtime hosted_k3s_agent_healthcheck_uses_lease_grace_window_for_transient_failures -- --nocapture, proof: ac-1.log -->
- [x] [SRS-02/AC-02] Hosted wedge detection no longer classifies healthy long-lived machines as `guest` wedged solely because guest refresh age is missing. <!-- [SRS-02/AC-02] verify: cargo test -p port-runtime effective_recovery_wedge_does_not_mark_missing_guest_refresh_age_as_guest_wedge -- --nocapture, proof: ac-2.log -->
- [x] [SRS-03/AC-03] Recovery action selection respects the settle window while recovery is in progress. <!-- [SRS-03/AC-03] verify: cargo test -p port-runtime recovery_decision_ -- --nocapture && cargo test -p port-runtime reconcile_machine_recovery_clears_in_progress_when_wedge_is_gone -- --nocapture, proof: ac-3.log -->
- [x] [SRS-04/AC-04] The machine wedge endpoint returns explicit hosted runtime evidence that explains the wedge classification and recovery posture. <!-- [SRS-04/AC-04] verify: cargo test -p port-runtime machine_wedge_status_serde_round_trips_with_defaults_skipped_on_wire -- --nocapture, proof: ac-4.log -->
- [x] [SRS-NFR-01/AC-05] The new wedge evidence fields remain additive and machine-readable for downstream consumers. <!-- [SRS-NFR-01/AC-05] verify: just check, proof: ac-5.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VHUlRkBvx/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VHUlRkBvx/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VHUlRkBvx/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VHUlRkBvx/EVIDENCE/ac-4.log)
- [ac-5.log](../../../../stories/VHUlRkBvx/EVIDENCE/ac-5.log)



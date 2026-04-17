# VOYAGE REPORT: Cluster Aggregate Wedge Field Threading

## Voyage Metadata
- **ID:** VH0mjMP8p
- **Epic:** VH0mU3DbK
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 1/1 stories complete

## Implementation Narrative
### Thread Wedge Fields Onto HostedK3sMachineTruth
- **ID:** VH0oGGkcz
- **Status:** done

#### Summary
Extend `HostedK3sMachineTruth` with the six per-machine wedge/recovery fields and populate them inside `hosted_k3s_machine_truth` via a new dedicated control-plane wedge route. Concretely:

1. Add the six fields (`guest_refresh_age_seconds`, `wedged_since_unix_s`, `wedge_class`, `recovery_attempts`, `last_recovery_action`, `recovery_state`) to `HostedK3sMachineTruth` with serde defaults so older payloads still decode unchanged.
2. Add a new read-only `MachineWedgeStatus` payload and a dedicated `GET /v1/machines/<name>/wedge` route on the control plane. The handler reads `wedge_state` directly from `ControlPlaneStateInner` and loads any persisted recovery record from the resolved per-machine runtime root — it does NOT proxy to the node agent and does NOT issue any guest operation.
3. Use the route from `hosted_k3s_machine_truth` (best-effort) so `port cluster status --format json` carries wedge state on the cluster aggregate without changing the existing guest-operation traffic profile.
4. Render the new fields per machine in `print_cluster_status_report`.

The dedicated wedge route exists because reusing `machine_status` per machine would issue extra `ManagedService::List` operations through vsock to the guest (as `inspect_machine` populates managed-service runtime state), which is wasteful and forces test fixtures to model that traffic. The wedge route stays read-only against control-plane state, mirroring the consumer-side cluster-wedge probe contract.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `HostedK3sMachineTruth` carries `guest_refresh_age_seconds`, `wedged_since_unix_s`, `wedge_class`, `recovery_attempts`, `last_recovery_action`, and `recovery_state` with `#[serde(default, skip_serializing_if = ...)]` and round-trips through serde unchanged when fields are absent. The new `MachineWedgeStatus` payload returned by the wedge route follows the same serde-skip-empty convention. <!-- [SRS-01/AC-01] verify: cargo test -p port-runtime --lib hosted_k3s_machine_truth_serde_round_trips_with_wedge_fields, proof: ac-1.log -->
- [x] [SRS-02/AC-02] `hosted_k3s_machine_truth` populates the new fields per machine from the dedicated control-plane `GET /v1/machines/<name>/wedge` route via `hosted_control_plane_machine_wedge`; if the route call errors, the new fields stay at serde defaults and the rest of the truth row builds unchanged. The wedge handler reads `wedge_state` and the per-machine recovery record without proxying to the node agent or issuing any guest operation. <!-- [SRS-02/AC-02] verify: cargo test -p port-runtime --lib hosted_k3s_machine_truth_leaves_wedge_fields_default_when_wedge_route_unreachable, proof: ac-2.log -->
- [x] [SRS-03/AC-03] `print_cluster_status_report` emits per-machine `guest refresh age seconds:`, `wedged since:`, and `wedge class:` lines that mirror the `print_machine_status` rendering, and prints `(none)` for absent values. <!-- [SRS-03/AC-03] verify: cargo test -p port --lib print_cluster_status_report_renders_wedge_fields, proof: ac-3.log -->
- [x] [SRS-04/AC-04] The new fields and the wedge route are covered by serde round-trip, population, and render tests; existing hosted cluster-status integration tests in `port-cli` continue to pass without changing their expected guest-operation sequences. <!-- [SRS-04/AC-04] verify: cargo test -p port --test machine_commands cli_cluster_status, proof: ac-4.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VH0oGGkcz/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VH0oGGkcz/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VH0oGGkcz/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VH0oGGkcz/EVIDENCE/ac-4.log)



# VOYAGE REPORT: Define Builder And Promotion Runtime Class Contracts

## Voyage Metadata
- **ID:** VGYQ4zrrX
- **Epic:** VGYFpewpf
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 2/2 stories complete

## Implementation Narrative
### Model Machine Runtime Class Contracts For Builder Lanes
- **ID:** VGYQnafG0
- **Status:** done

#### Summary
Add the shared runtime-class contract to Port's machine model so
`workspace-scratch-builder` becomes an explicit, validated execution lane with
declared writable roots and bounded trust posture.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `MachineSpec` can serialize and deserialize an explicit runtime-class declaration instead of relying on machine naming or comments. Verified by `cargo test -q -p port-model workspace_scratch_runtime_class_round_trips -- --nocapture` in `EVIDENCE/ac-1.roundtrip.log`. <!-- verify: command, SRS-01:start:end, proof: ac-1.roundtrip.log -->
- [x] [SRS-02/AC-02] Port defines a canonical `workspace-scratch-builder` runtime-class contract that records its workspace-bound writable-state categories and explicitly untrusted posture. Verified by `cargo test -q -p port-model workspace_scratch_runtime_class_round_trips -- --nocapture` in `EVIDENCE/ac-2.builder-contract.log`. <!-- verify: command, SRS-02:start:end, proof: ac-2.builder-contract.log -->
- [x] [SRS-04/AC-03] Config validation rejects contradictory builder runtime-class declarations, including missing writable-state metadata or publish-trusted posture on the scratch lane. Verified by `cargo test -q -p port-model workspace_scratch_runtime_class -- --nocapture` in `EVIDENCE/ac-3.validation.log`. <!-- verify: command, SRS-04:start:end, proof: ac-3.validation.log -->

#### Verified Evidence
- [ac-1.roundtrip.log](../../../../stories/VGYQnafG0/EVIDENCE/ac-1.roundtrip.log)
- [ac-2.builder-contract.log](../../../../stories/VGYQnafG0/EVIDENCE/ac-2.builder-contract.log)
- [ac-3.validation.log](../../../../stories/VGYQnafG0/EVIDENCE/ac-3.validation.log)

### Surface Builder Runtime Class Identity In Machine Output
- **ID:** VGYQnawG1
- **Status:** done

#### Summary
Carry the new runtime-class contract through Port's machine launch and status
surfaces so operators and downstream tooling can inspect which builder lane ran
and which trust posture it carried.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] Machine-facing Port metadata carries runtime-class identity and posture for builder lanes instead of dropping that contract after config validation. Verified by `cargo test -q -p port-runtime machine_status_reports_runtime_paths_for_known_machine -- --nocapture` in `EVIDENCE/ac-1.runtime-status.log` and `cargo test -q -p port-runtime stop_machine_terminates_live_port_owned_process -- --nocapture` in `EVIDENCE/ac-2.runtime-stop.log`. <!-- verify: command, SRS-03:start:end, proof: ac-1.runtime-status.log, ac-2.runtime-stop.log -->
- [x] [SRS-NFR-01/AC-02] The surfaced runtime-class contract stays environment-agnostic so local and AWS lanes would report the same builder vocabulary. Verified by `cargo test -q -p port machine_status_render_includes_runtime_class_contract -- --nocapture` in `EVIDENCE/ac-3.cli-render.log`, which renders the same builder labels on a hosted `cloud-aws` sample status. <!-- verify: command, SRS-NFR-01:start:end, proof: ac-3.cli-render.log -->
- [x] [SRS-NFR-02/AC-03] CLI output makes the scratch-builder lane visibly untrusted and does not imply publish or admin rights. Verified by `cargo test -q -p port machine_status_render_includes_runtime_class_contract -- --nocapture` in `EVIDENCE/ac-3.cli-render.log`. <!-- verify: command, SRS-NFR-02:start:end, proof: ac-3.cli-render.log -->
- [x] [SRS-NFR-03/AC-04] Verification for this story includes targeted automated tests plus one operator-visible machine output proof. Verified in `EVIDENCE/ac-1.runtime-status.log`, `EVIDENCE/ac-2.runtime-stop.log`, and `EVIDENCE/ac-3.cli-render.log`. <!-- verify: command, SRS-NFR-03:start:end, proof: ac-1.runtime-status.log, ac-2.runtime-stop.log, ac-3.cli-render.log -->

#### Verified Evidence
- [ac-1.runtime-status.log](../../../../stories/VGYQnawG1/EVIDENCE/ac-1.runtime-status.log)
- [ac-2.runtime-stop.log](../../../../stories/VGYQnawG1/EVIDENCE/ac-2.runtime-stop.log)
- [ac-3.cli-render.log](../../../../stories/VGYQnawG1/EVIDENCE/ac-3.cli-render.log)



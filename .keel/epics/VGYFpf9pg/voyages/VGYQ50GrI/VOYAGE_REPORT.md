# VOYAGE REPORT: Prove Runtime Class Identity And Guard Rails

## Voyage Metadata
- **ID:** VGYQ50GrI
- **Epic:** VGYFpf9pg
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 1/1 stories complete

## Implementation Narrative
### Define Promotion Runner Runtime Class Guard Rails
- **ID:** VGYQnbJG2
- **Status:** done

#### Summary
Define the promotion-runner runtime class as a clean-room lane whose identity,
declared-input posture, and validation rules remain distinct from
`workspace-scratch-builder`.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Port models `blessed-closure-promotion-runner` as a dedicated runtime class rather than as a trust flag on `workspace-scratch-builder`. Verified by `cargo test -q -p port-model promotion_runner_runtime_class_round_trips -- --nocapture` in `EVIDENCE/ac-1.model.log`. <!-- verify: command, SRS-01:start:end, proof: ac-1.model.log -->
- [x] [SRS-02/AC-02] The promotion-runner contract declares clean-room and immutable-input posture explicitly. Verified by `cargo test -q -p port-model promotion_runner_runtime_class_round_trips -- --nocapture` in `EVIDENCE/ac-1.model.log`, which round-trips `state_isolation = "clean-room"` and the declared promotion inputs. <!-- verify: command, SRS-02:start:end, proof: ac-1.model.log -->
- [x] [SRS-03/AC-03] Port machine-facing proof surfaces carry promotion-runner identity and posture so downstream publication tooling can link runtime evidence to what ran. Verified by `cargo test -q -p port machine_status_render_includes_promotion_runner_contract -- --nocapture` in `EVIDENCE/ac-3.cli-render.log`. <!-- verify: command, SRS-03:start:end, proof: ac-3.cli-render.log -->
- [x] [SRS-04/AC-03] Config validation rejects promotion-runner declarations that try to reuse scratch writable state or creator credentials. Verified by `cargo test -q -p port-model promotion_runner_runtime_class -- --nocapture` in `EVIDENCE/ac-2.validation.log`. <!-- verify: command, SRS-04:start:end, proof: ac-2.validation.log -->
- [x] [SRS-NFR-03/AC-04] Verification includes at least one negative-path proof for a collapsed scratch/promotion trust boundary. Verified by `cargo test -q -p port-model promotion_runner_runtime_class -- --nocapture` in `EVIDENCE/ac-2.validation.log`, which exercises rejected workspace bindings and scratch writable roots on the promotion lane. <!-- verify: command, SRS-NFR-03:start:end, proof: ac-2.validation.log -->

#### Verified Evidence
- [ac-1.model.log](../../../../stories/VGYQnbJG2/EVIDENCE/ac-1.model.log)
- [ac-2.validation.log](../../../../stories/VGYQnbJG2/EVIDENCE/ac-2.validation.log)
- [ac-3.cli-render.log](../../../../stories/VGYQnbJG2/EVIDENCE/ac-3.cli-render.log)
- [ac-4.fmt-check.log](../../../../stories/VGYQnbJG2/EVIDENCE/ac-4.fmt-check.log)



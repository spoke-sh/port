---
# system-managed
id: VGYQnawG1
status: done
created_at: 2026-04-11T23:10:11
updated_at: 2026-04-11T23:26:37
# authored
title: Surface Builder Runtime Class Identity In Machine Output
type: feat
operator-signal:
scope: VGYFpewpf/VGYQ4zrrX
index: 2
started_at: 2026-04-11T23:19:40
completed_at: 2026-04-11T23:26:37
---

# Surface Builder Runtime Class Identity In Machine Output

## Summary

Carry the new runtime-class contract through Port's machine launch and status
surfaces so operators and downstream tooling can inspect which builder lane ran
and which trust posture it carried.

## Acceptance Criteria

- [x] [SRS-03/AC-01] Machine-facing Port metadata carries runtime-class identity and posture for builder lanes instead of dropping that contract after config validation. Verified by `cargo test -q -p port-runtime machine_status_reports_runtime_paths_for_known_machine -- --nocapture` in `EVIDENCE/ac-1.runtime-status.log` and `cargo test -q -p port-runtime stop_machine_terminates_live_port_owned_process -- --nocapture` in `EVIDENCE/ac-2.runtime-stop.log`. <!-- verify: command, SRS-03:start:end, proof: ac-1.runtime-status.log, ac-2.runtime-stop.log -->
- [x] [SRS-NFR-01/AC-02] The surfaced runtime-class contract stays environment-agnostic so local and AWS lanes would report the same builder vocabulary. Verified by `cargo test -q -p port machine_status_render_includes_runtime_class_contract -- --nocapture` in `EVIDENCE/ac-3.cli-render.log`, which renders the same builder labels on a hosted `cloud-aws` sample status. <!-- verify: command, SRS-NFR-01:start:end, proof: ac-3.cli-render.log -->
- [x] [SRS-NFR-02/AC-03] CLI output makes the scratch-builder lane visibly untrusted and does not imply publish or admin rights. Verified by `cargo test -q -p port machine_status_render_includes_runtime_class_contract -- --nocapture` in `EVIDENCE/ac-3.cli-render.log`. <!-- verify: command, SRS-NFR-02:start:end, proof: ac-3.cli-render.log -->
- [x] [SRS-NFR-03/AC-04] Verification for this story includes targeted automated tests plus one operator-visible machine output proof. Verified in `EVIDENCE/ac-1.runtime-status.log`, `EVIDENCE/ac-2.runtime-stop.log`, and `EVIDENCE/ac-3.cli-render.log`. <!-- verify: command, SRS-NFR-03:start:end, proof: ac-1.runtime-status.log, ac-2.runtime-stop.log, ac-3.cli-render.log -->

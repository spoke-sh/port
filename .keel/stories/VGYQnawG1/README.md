---
# system-managed
id: VGYQnawG1
status: backlog
created_at: 2026-04-11T23:10:11
updated_at: 2026-04-11T23:11:44
# authored
title: Surface Builder Runtime Class Identity In Machine Output
type: feat
operator-signal:
scope: VGYFpewpf/VGYQ4zrrX
index: 2
---

# Surface Builder Runtime Class Identity In Machine Output

## Summary

Carry the new runtime-class contract through Port's machine launch and status
surfaces so operators and downstream tooling can inspect which builder lane ran
and which trust posture it carried.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] Machine-facing Port metadata carries runtime-class identity and posture for builder lanes instead of dropping that contract after config validation. <!-- verify: automated, SRS-03:start:end -->
- [ ] [SRS-NFR-01/AC-02] The surfaced runtime-class contract stays environment-agnostic so local and AWS lanes would report the same builder vocabulary. <!-- verify: automated, SRS-NFR-01:start:end -->
- [ ] [SRS-NFR-02/AC-03] CLI output makes the scratch-builder lane visibly untrusted and does not imply publish or admin rights. <!-- verify: manual, SRS-NFR-02:start:end -->
- [ ] [SRS-NFR-03/AC-04] Verification for this story includes targeted automated tests plus one operator-visible machine output proof. <!-- verify: manual, SRS-NFR-03:start:end -->

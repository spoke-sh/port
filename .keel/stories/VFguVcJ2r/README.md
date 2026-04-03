---
# system-managed
id: VFguVcJ2r
status: backlog
created_at: 2026-04-02T19:29:29
updated_at: 2026-04-02T19:31:27
# authored
title: Define Guest Session Identity Surface
type: feat
operator-signal:
scope: VFgtgGEog/VFgu7Bd7U
index: 1
---

# Define Guest Session Identity Surface

## Summary

Define the implementation-ready guest-session identity and driver metadata
contract for hosted guest-backed shell flows so upstream systems can audit one
Port-owned shell driver on the verified AWS PVM lane.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end, proof: ac-1.log -->
- [ ] [SRS-01/AC-01] Planning artifacts define the stable guest-session identifier for hosted guest-backed `exec`, `pty`, and `forward`. <!-- [SRS-01/AC-01] verify: manual, proof: ac-1.log -->
<!-- verify: manual, SRS-02:start:end, proof: ac-2.log -->
- [ ] [SRS-02/AC-02] Planning artifacts define one audited driver metadata contract for hosted guest-backed `exec`, `pty`, and `forward`. <!-- [SRS-02/AC-02] verify: manual, proof: ac-2.log -->
<!-- verify: manual, SRS-03:start:end, proof: ac-3.log -->
- [ ] [SRS-03/AC-03] The contract keeps session identity and driver metadata on canonical Port surfaces rather than a creator-specific API. <!-- [SRS-03/AC-03] verify: manual, proof: ac-3.log -->
<!-- verify: manual, SRS-04:start:end, proof: ac-4.log -->
- [ ] [SRS-04/AC-04] The contract makes unsupported or missing session metadata fail explicitly with no ambiguous or anonymous fallback. <!-- [SRS-04/AC-04] verify: manual, proof: ac-4.log -->
<!-- verify: manual, SRS-NFR-01:start:end, proof: ac-5.log -->
- [ ] [SRS-NFR-01/AC-05] Stability expectations are explicit enough to guide the first execution stories and downstream audit consumers. <!-- [SRS-NFR-01/AC-05] verify: manual, proof: ac-5.log -->
<!-- verify: manual, SRS-NFR-02:start:end, proof: ac-6.log -->
- [ ] [SRS-NFR-02/AC-06] Proof obligations are explicit enough to guide the first execution stories and downstream audit consumers. <!-- [SRS-NFR-02/AC-06] verify: manual, proof: ac-6.log -->

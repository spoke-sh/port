---
# system-managed
id: VGgBIcexb
status: done
created_at: 2026-04-13T06:58:47
updated_at: 2026-04-13T07:02:30
# authored
title: Upgrade Keel Flake Input
type: chore
operator-signal:
started_at: 2026-04-13T06:58:50
completed_at: 2026-04-13T07:02:30
---

# Upgrade Keel Flake Input

## Summary

Advance the repository's `keel` flake input to a newer upstream revision and
prove the exported `keel` package still evaluates and builds through the Port
flake after the lockfile change.

## Acceptance Criteria

<!-- verify: command, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-01/AC-01] `flake.lock` advances the root `keel` input to the intended upstream revision without breaking the follow-edges used by `atxt` and `paddles`. <!-- [SRS-01/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/VGgBIcexb/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-02/AC-02] The Port flake still evaluates and builds the exported `keel` package after the upgrade. <!-- [SRS-02/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/VGgBIcexb/verify-ac-2.sh, proof: ac-2.log -->

## Proof

- AC-01: `EVIDENCE/ac-1.log` records the upgraded `flake.lock` revision
  `65af71bb72f871fcd7249913a9580d8cfb1fbf2b` plus the preserved `keel`
  follow-edges from the root, `atxt`, and `paddles` inputs.
- AC-02: `EVIDENCE/ac-2.log` records a successful `nix build .#keel --no-link`
  and the built binary reporting `keel 0.2.1`.

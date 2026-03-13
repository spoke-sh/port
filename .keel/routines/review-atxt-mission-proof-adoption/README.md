---
id: review-atxt-mission-proof-adoption
title: Review ATXT Mission Proof Adoption
cadence:
  cron: 0 9 1 * *
  timezone: America/Los_Angeles
target-scope: VDaiFfFPe
created_at: 2026-03-12T17:52:14
updated_at: 2026-03-12T17:52:14
---

# Blueprint

- Trigger: monthly review of whether `atxt` is ready to replace or augment the
  current `vhs`-backed human-reviewable recording path for the repo-level
  proof surface (`just mission` today, `just screen` after the planned rename).
- Review the current proof contract first:
  - the repo-level proof surface should prove Port can host a minimal HTTP
    application, curl it successfully from the host, and surface a
    human-reviewable recording.
  - The current acceptable recorder path is `vhs` or renderer-backed `.gif` /
    `.cast` evidence.
- Assess `atxt` readiness in the current repository environment:
  - verify installation and shell compatibility in `nix develop`
  - verify it can capture the canonical app-hosting proof legibly in a
    human-reviewable terminal artifact
  - identify blockers if it still cannot replace the current recorder path
- Exit criteria:
  - if `atxt` is ready, create and activate a dedicated mission to migrate the
    canonical screen recording path to `atxt`, then plan and implement the
    first scoped slice
  - if `atxt` is not ready, record the blocker, refresh the proof of why the
    current recorder remains canonical, and leave the routine in place for the
    next review window

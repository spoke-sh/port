---
# system-managed
id: VIl0aVbn5
status: done
created_at: 2026-05-05T07:45:22
updated_at: 2026-05-05T08:01:42
# authored
title: Release Port Version 0 1 0
type: chore
operator-signal:
started_at: 2026-05-05T07:45:24
submitted_at: 2026-05-05T08:01:40
completed_at: 2026-05-05T08:01:42
---

# Release Port Version 0 1 0

## Summary

Prepare and publish Port's first installable release as `v0.1.0` after the
hosted infra deployment remained stable overnight. The release is tag-driven
through the existing cargo-dist workflow, so this story records the release
checklist evidence, board hygiene, and tag boundary.

## Acceptance Criteria

- [x] [SRS-01/AC-01] Release metadata and support-boundary docs are aligned for `v0.1.0`: the workspace version is `0.1.0`, no existing `v0.1.0` tag is present before release, and `RELEASE.md`, `docs/install.md`, and `docs/avf.md` keep the shipped target matrix explicit. <!-- verify: manual, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-02] The canonical release validation path passes for the current release commit: mission context, workspace tests, doctests, Linux package proof, `dist plan`, packaged `port doctor`, and `keel doctor`. <!-- verify: manual, SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-03/AC-03] The `v0.1.0` release boundary is committed and tagged from the validated commit without leaving uncommitted release work in the repository. <!-- verify: manual, SRS-03:start:end, proof: ac-3.log -->

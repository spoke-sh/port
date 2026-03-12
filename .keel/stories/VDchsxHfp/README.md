---
id: VDchsxHfp
title: Publish Installable Support Matrix And Release Contract
type: feat
status: backlog
created_at: 2026-03-11T21:16:30
updated_at: 2026-03-11T21:19:02
scope: VDcT0vaPb/VDchK6xzs
index: 3
---

# Publish Installable Support Matrix And Release Contract

## Summary

Publish the first installable Linux and macOS support matrix and rewrite the
release checklist so the package workflow, platform boundaries, and canonical
validation path are explicit in the operator-facing docs.

## Acceptance Criteria

<!-- verify: command, SRS-01:start:end, proof: ac-1.log -->
- [ ] [SRS-01/AC-01] `README.md`, `RELEASE.md`, and the install-focused docs publish the first supported Linux and macOS targets, their canonical package artifact, and the unsupported-environment boundary for this slice. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "Supported Targets|canonical package|WSL|remote Linux host|macOS" README.md RELEASE.md docs', proof: ac-1.log -->

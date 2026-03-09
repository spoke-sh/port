---
id: 1vzWCJ000
title: Route Artifact Push And Pull Through Hosted Backend
type: feat
status: backlog
created_at: 2026-03-09T01:42:43
updated_at: 2026-03-09T01:45:28
scope: 1vzW8e000/1vzW9Q000
---

# Route Artifact Push And Pull Through Hosted Backend

## Summary

Route the canonical `port artifacts push|pull` commands through the hosted
artifact backend so operators use the existing CLI vocabulary while Port prints
deterministic backend and path details for the selected variant.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] `port artifacts push` routes to the configured hosted backend and uploads the selected artifact variant through the hosted transport instead of the file-system backend.
- [ ] [SRS-03/AC-02] `port artifacts pull` routes to the hosted backend and materializes the selected variant into both the canonical local output path and the cache path.
- [ ] [SRS-03/AC-03] Canonical CLI output for hosted push and pull includes artifact selector, backend, local path, cache path, and hosted store path detail without introducing a second artifact command family.

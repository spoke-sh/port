---
id: 1vzWCJ000
title: Route Artifact Push And Pull Through Hosted Backend
type: feat
status: in-progress
created_at: 2026-03-09T01:42:43
updated_at: 2026-03-09T02:14:33
scope: 1vzW8e000/1vzW9Q000
started_at: 2026-03-09T02:14:33
---

# Route Artifact Push And Pull Through Hosted Backend

## Summary

Route the canonical `port artifacts push|pull` commands through the hosted
artifact backend so operators use the existing CLI vocabulary while Port prints
deterministic backend and path details for the selected variant.

## Acceptance Criteria

<!-- verify: command, SRS-03:start, proof: ac-1.log -->
- [ ] [SRS-03/AC-01] `port artifacts push` routes to the configured hosted backend and uploads the selected artifact variant through the hosted transport instead of the file-system backend. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime push_and_pull_artifact_round_trip_through_live_hosted_backend && cargo test -q -p port-cli --test artifact_commands cli_artifact_push_and_pull_round_trip_through_hosted_backend', proof: ac-2.log -->
<!-- verify: command, SRS-03:continues, proof: ac-2.log -->
- [ ] [SRS-03/AC-02] `port artifacts pull` routes to the hosted backend and materializes the selected variant into both the canonical local output path and the cache path. <!-- [SRS-03/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime push_and_pull_artifact_round_trip_through_live_hosted_backend && cargo test -q -p port-cli --test artifact_commands cli_artifact_push_and_pull_round_trip_through_hosted_backend', proof: ac-2.log -->
<!-- verify: command, SRS-03:end, proof: ac-3.log -->
- [ ] [SRS-03/AC-03] Canonical CLI output for hosted push and pull includes artifact selector, backend, local path, cache path, and hosted store path detail without introducing a second artifact command family. <!-- [SRS-03/AC-03] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-cli --test artifact_commands cli_artifact_push_and_pull_round_trip_through_hosted_backend', proof: ac-3.log -->

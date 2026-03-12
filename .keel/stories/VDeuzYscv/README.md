---
id: VDeuzYscv
title: Implement SSH Machine Lifecycle Routing
type: feat
status: backlog
created_at: 2026-03-12T06:21:06
updated_at: 2026-03-12T06:23:42
operator-signal: 
scope: VDcStPolu/VDeuazAgk
index: 2
---

# Implement SSH Machine Lifecycle Routing

## Summary

Implement the first bounded SSH-first machine lifecycle path so canonical
`launch`, `status`, and `stop` verbs can target a remote Linux host without
forking the CLI or hiding route ownership.

## Acceptance Criteria

- [ ] [SRS-03/AC-01] `port machine launch`, `status`, and `stop` route through an SSH-managed remote Linux host for the first supported lifecycle slice. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q cli_ssh_machine_launch_status_and_stop_round_trip', proof: ac-1.log -->
- [ ] [SRS-04/AC-02] SSH lifecycle output and failure paths keep machine, host, provider, route, and ownership context explicit. <!-- [SRS-04/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q cli_ssh_machine_route_context', proof: ac-2.log -->

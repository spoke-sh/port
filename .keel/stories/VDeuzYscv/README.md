---
id: VDeuzYscv
title: Implement SSH Machine Lifecycle Routing
type: feat
status: done
created_at: 2026-03-12T06:21:06
updated_at: 2026-03-12T07:00:50
operator-signal: 
scope: VDcStPolu/VDeuazAgk
index: 2
started_at: 2026-03-12T06:49:04
completed_at: 2026-03-12T07:00:50
---

# Implement SSH Machine Lifecycle Routing

## Summary

Implement the first bounded SSH-first machine lifecycle path so canonical
`launch`, `status`, and `stop` verbs can target a remote Linux host without
forking the CLI or hiding route ownership.

## Acceptance Criteria

<!-- verify: command, SRS-03:start:end, proof: ac-1.log -->
- [x] [SRS-03/AC-01] `port machine launch`, `status`, and `stop` route through an SSH-managed remote Linux host for the first supported lifecycle slice. <!-- [SRS-03/AC-01] verify: cargo test -q --test machine_commands cli_ssh_machine_launch_status_and_stop_round_trip, proof: ac-1.log -->
<!-- verify: command, SRS-04:start:end, proof: ac-2.log -->
- [x] [SRS-04/AC-02] SSH lifecycle output and failure paths keep machine, host, provider, route, and ownership context explicit. <!-- [SRS-04/AC-02] verify: cargo test -q --test machine_commands cli_ssh_machine_route_context, proof: ac-2.log -->

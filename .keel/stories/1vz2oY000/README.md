---
id: 1vz2oY000
title: Add Machine Inventory Status And Stop
type: feat
status: backlog
created_at: 2026-03-07T17:20:14
updated_at: 2026-03-07T17:24:27
scope: 1vz2eV000/1vz2ky000
---

# Add Machine Inventory Status And Stop

## Summary

Add the first real machine lifecycle surfaces Port is missing: local inventory,
status, and stop commands backed by runtime manifests, pid inspection, and
coherent CLI output.

## Acceptance Criteria

- [ ] [SRS-02/AC-01] `port machine list` enumerates machines under the selected runtime root and reports their lifecycle state from manifests plus live process inspection.
- [ ] [SRS-02/AC-02] `port machine status --machine <name>` prints actionable runtime metadata including liveness, pid, runtime paths, and troubleshooting log references.
- [ ] [SRS-02/AC-03] `port machine stop --machine <name>` safely stops a Port-owned local machine and reports the resulting lifecycle outcome through the canonical CLI.
- [ ] [SRS-03/AC-04] Missing, stale, or malformed runtime state produces explicit diagnostics instead of silent skips or ambiguous failures.

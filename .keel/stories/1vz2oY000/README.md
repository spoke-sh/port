---
id: 1vz2oY000
title: Add Machine Inventory Status And Stop
type: feat
status: in-progress
created_at: 2026-03-07T17:20:14
updated_at: 2026-03-07T17:40:53
scope: 1vz2eV000/1vz2ky000
started_at: 2026-03-07T17:40:53
---

# Add Machine Inventory Status And Stop

## Summary

Add the first real machine lifecycle surfaces Port is missing: local inventory,
status, and stop commands backed by runtime manifests, pid inspection, and
coherent CLI output.

## Acceptance Criteria

<!-- verify: manual, SRS-02:start:end, proof: ac-1.log-->
- [ ] [SRS-02/AC-01] `port machine list` enumerates machines under the selected runtime root and reports their lifecycle state from manifests plus live process inspection. <!-- [SRS-02/AC-01] verify: nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime list_machines_reports_running_stale_and_malformed_runtime_entries && nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-cli help_includes_primary_surfaces, proof: ac-1.log-->
<!-- verify: manual, SRS-02:start:end, proof: ac-2.log-->
- [ ] [SRS-02/AC-02] `port machine status --machine <name>` prints actionable runtime metadata including liveness, pid, runtime paths, and troubleshooting log references. <!-- [SRS-02/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo run -p port-cli -- --config examples/port.toml machine status --machine demo --runtime-root /tmp/port-status-proof', proof: ac-2.log-->
<!-- verify: manual, SRS-02:start:end, proof: ac-3.log-->
- [ ] [SRS-02/AC-03] `port machine stop --machine <name>` safely stops a Port-owned local machine and reports the resulting lifecycle outcome through the canonical CLI. <!-- [SRS-02/AC-03] verify: nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime stop_machine_terminates_live_port_owned_process && nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-cli parses_machine_lifecycle_arguments, proof: ac-3.log-->
<!-- verify: manual, SRS-03:start:end, proof: ac-4.log-->
- [ ] [SRS-03/AC-04] Missing, stale, or malformed runtime state produces explicit diagnostics instead of silent skips or ambiguous failures. <!-- [SRS-03/AC-04] verify: nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime machine_status_reports_missing_and_malformed_runtime_state, proof: ac-4.log-->

---
# system-managed
id: VH0oGGkcz
status: icebox
created_at: 2026-04-16T19:39:00
updated_at: 2026-04-16T19:39:00
# authored
title: Thread Wedge Fields Onto HostedK3sMachineTruth
type: feat
operator-signal:
scope: VH0mU3DbK/VH0mjMP8p
index: 1
---

# Thread Wedge Fields Onto HostedK3sMachineTruth

## Summary

Extend `HostedK3sMachineTruth` (`crates/port-runtime/src/lib.rs:265`) with the six per-machine wedge/recovery fields that already exist on `MachineStatus`, and populate them inside `hosted_k3s_machine_truth` via the live `machine_status` call so `port cluster status --format json` carries the wedge state on the cluster aggregate. Mirror the per-machine HTTP pattern that `hosted_k3s_managed_service_truth` already uses for `list_machine_services`. Also extend `print_cluster_status_report` so the text output mirrors the JSON shape.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] `HostedK3sMachineTruth` carries `guest_refresh_age_seconds`, `wedged_since_unix_s`, `wedge_class`, `recovery_attempts`, `last_recovery_action`, and `recovery_state` with `#[serde(default, skip_serializing_if = ...)]` and round-trips through serde unchanged when fields are absent. <!-- [SRS-01/AC-01] verify: cargo test -p port-runtime hosted_k3s_machine_truth_serde, proof: ac-1.log -->
- [ ] [SRS-02/AC-02] `hosted_k3s_machine_truth` populates the new fields per machine from `machine_status(config, runtime_root, machine_name)`; if the call errors, the new fields stay at serde defaults and the rest of the truth row builds unchanged. <!-- [SRS-02/AC-02] verify: cargo test -p port-runtime hosted_k3s_machine_truth_populates_wedge_fields, proof: ac-2.log -->
- [ ] [SRS-03/AC-03] `print_cluster_status_report` emits per-machine `guest refresh age seconds:`, `wedged since:`, and `wedge class:` lines that mirror the `print_machine_status` rendering, and prints `(none)` for absent values. <!-- [SRS-03/AC-03] verify: cargo test -p port-cli print_cluster_status_report_renders_wedge_fields, proof: ac-3.log -->

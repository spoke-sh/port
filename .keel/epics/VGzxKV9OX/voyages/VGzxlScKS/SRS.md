# Wedge Detector And Cluster Status Fields - SRS

## Summary

Epic: VGzxKV9OX
Goal: Introduce a configurable wedge detector that consumes both refresh_age_seconds (node-side) and guest_refresh_age_seconds (guest-side) and surfaces wedged_since, wedge_class, recovery_attempts, last_recovery_action, and recovery_state in port cluster status --format json. No recovery actions yet.

## Scope

### In Scope

- [SCOPE-04] A detection-only configuration block in `port.toml` exposing `node_trigger_refresh_age_seconds` and `guest_trigger_refresh_age_seconds` (with conservative defaults) so operators can tune thresholds without touching recovery actions.
- [SCOPE-04] A detector task in the hosted control plane that evaluates both heartbeat ages for every machine at a regular interval; sets `wedged_since` (timestamp of first observed stale read) and `wedge_class` (`"guest"` or `"node"`) when the matching trigger fires; clears both when the trigger conditions no longer hold.
- [SCOPE-03] Surface `wedged_since: Option<timestamp>` and `wedge_class: Option<"guest" | "node">` per machine in `port cluster status --format json` and the machine status contract.

### Out of Scope

- [SCOPE-06] Any recovery action, counter, or `recovery_state` value — owned by epic VGzxMc4G4.
- [SCOPE-07] Alerting, dashboards, or UI surfaces.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `port.toml` grows a `[clusters.<name>.detection]` block with `node_trigger_refresh_age_seconds` and `guest_trigger_refresh_age_seconds`; missing blocks yield safe conservative defaults, and a doctor/validation path rejects clearly invalid values (zero, negative). | SCOPE-04 | FR-01 | unit |
| SRS-02 | The hosted control plane runs a detector task that, per machine per interval, evaluates `refresh_age_seconds > node_threshold` and `guest_refresh_age_seconds > guest_threshold`. When either trigger fires, the detector sets `wedged_since = <now>` (only on the first observed stale read) and `wedge_class` accordingly; when both trigger conditions are false again, it clears both fields. | SCOPE-04 | FR-01 | unit |
| SRS-03 | When both `guest_refresh_age_seconds` and `refresh_age_seconds` are stale on the same machine, the detector prefers `wedge_class = "node"` because tier-1/tier-2 recovery cannot act through a silent node-agent anyway; the chosen precedence is covered by a test. | SCOPE-04 | FR-01 | unit |
| SRS-04 | `port cluster status --format json` and the machine status contract expose `wedged_since: Option<u64>` (unix seconds) and `wedge_class: Option<String>` per machine, skipped when `None`; the CLI human-readable render includes them with a `(none)` fallback. | SCOPE-03 | FR-01 | integration |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The detector reads state but does not mutate any machine or guest runtime; fault-injection tests assert no `port machine stop/launch` side effects while the detector is running. | SCOPE-04 | NFR-01 | unit |
| SRS-NFR-02 | Detector evaluation runs on an independent interval task rather than being invoked synchronously from any other loop; the task holds its own `tokio::time::interval` handle and its tick frequency is bounded by config, not by external traffic. | SCOPE-04 | NFR-01 | unit |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

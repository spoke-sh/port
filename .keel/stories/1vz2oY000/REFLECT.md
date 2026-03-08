---
created_at: 2026-03-07T17:49:41
---

# Reflection - Add Machine Inventory Status And Stop

## Knowledge

### 1vz3Mb000: Runtime Lifecycle Should Key Off Runtime State, Not Config
| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When Port adds lifecycle or inspection commands that must keep working after launch, across config drift, or under a future hosted control plane |
| **Insight** | `launch` is model-backed, but `list`, `status`, and `stop` should key off runtime manifests and PID inspection instead of reloading the machine model. That keeps lifecycle commands usable after a VM already exists and matches the control-plane direction for hosted Port. |
| **Suggested Action** | Treat runtime-root inspection data as the source of truth for post-launch lifecycle commands, and only require the model for launch-time validation or artifact resolution. |
| **Applies To** | crates/port-runtime/**, crates/port-cli/**, docs/operators.md, README.md |
| **Observed At** | 2026-03-08T01:55:00Z |
| **Score** | 0.91 |
| **Confidence** | 0.95 |
| **Applied** | yes |

## Observations

- The runtime already had enough durable state to ship useful lifecycle
  surfaces. `manifest.json`, `firecracker.pid`, and `/proc` inspection were
  sufficient for `list`, `status`, and `stop` without waiting for a daemon or
  Firecracker's HTTP API.
- Status quality improved once the output included concrete runtime paths.
  Showing the config path, manifest, pid file, and console/log files makes the
  CLI useful as an operator surface rather than only as a wrapper around helper
  functions.
- The right operator boundary was `launch` versus everything after launch.
  Requiring `--config` for launch still makes sense, but lifecycle commands are
  cleaner and more future-proof when they operate directly on the runtime root.
- Live CLI proof mattered here. The end-to-end `launch -> list -> status ->
  stop -> status` transcript caught environment assumptions and validated that
  the stop path leaves deterministic relaunch state behind.

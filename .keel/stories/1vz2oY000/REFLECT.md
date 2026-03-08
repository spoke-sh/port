---
created_at: 2026-03-07T17:49:41
---

# Reflection - Add Machine Inventory Status And Stop

## Knowledge

- [1vz3Mb000](../../knowledge/1vz3Mb000.md) Runtime Lifecycle Should Key Off Runtime State, Not Config

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

---
created_at: 2026-03-08T19:41:25
---

# Reflection - Implement Hosted Detached Forward Inventory

## Knowledge

### 1vzQL2000: Detached Forward Runtime Helpers Must Not Assume `current_exe` Is The Port CLI
| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Launching detached forward daemons from shared runtime code that also runs under library tests and hosted node-agent servers |
| **Insight** | `std::env::current_exe()` can resolve to a Rust test harness instead of the `port` CLI binary, so detached child-process launch must prefer an explicit or workspace `port` binary path before falling back to the current executable. |
| **Suggested Action** | Keep detached helper launchers behind a resolver that checks `PORT_DETACHED_FORWARD_EXECUTABLE` and the workspace `target/debug/port` path before using `current_exe()`. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, detached runtime helpers, hosted node-agent tests |
| **Linked Knowledge IDs** | |
| **Observed At** | 2026-03-08T19:41:25Z |
| **Score** | 0.89 |
| **Confidence** | 0.97 |
| **Applied** | yes |

## Observations

The node-owned model held up once the lifecycle helpers moved into
`port-runtime`. The hosted control plane and node agent could reuse the same
runtime-root manifest directory the local lane already exposed through
`machine monitor` and `top`, which kept the design small.

Two concrete issues surfaced during implementation. Axum route segments cannot
mix `{param}` with a literal suffix like `:stop`, so the detached stop route
had to cut over to `/guest:forward:detached/{name}/stop`. Also, storing the
localized config file beside forward manifests required the manifest loader to
filter for `.json` files or list operations would report malformed entries.

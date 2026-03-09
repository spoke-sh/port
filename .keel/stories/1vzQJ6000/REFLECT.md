---
created_at: 2026-03-08T19:41:25
---

# Reflection - Implement Hosted Detached Forward Inventory

## Knowledge

- [1vzQL2000](../../knowledge/1vzQL2000.md) Detached Forward Runtime Helpers Must Not Assume `current_exe` Is The Port CLI

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

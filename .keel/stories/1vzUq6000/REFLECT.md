---
created_at: 2026-03-09T00:39:32
---

# Reflection - Persist Hosted Registration And Freshness

## Knowledge

- [1vzVJp000](../../knowledge/1vzVJp000.md) Isolate Default Hosted Test State From Demo Runtime Files

## Observations

The durable registration path itself was stable once the targeted persistence tests were in place. The harder issue was test isolation: helper readiness probes and reused `demo` runtime files were leaking state into unrelated hosted proxy tests, which made isolated reruns fail even when the full suite passed.

Fixing the helper layer improved confidence in the product change. A dedicated mock readiness route stopped probes from polluting request assertions, node-agent readiness now waits for any HTTP response rather than only successful status, and the default static-binding control-plane helper now clears stale `demo` registry state before serving.

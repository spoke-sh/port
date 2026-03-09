---
created_at: 2026-03-09T00:39:32
---

# Reflection - Persist Hosted Registration And Freshness

## Knowledge

### 1vzVJp000: Isolate Default Hosted Test State From Demo Runtime Files
| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | When hosted control-plane tests use the default `demo` control plane and static node bindings. |
| **Insight** | Persisted hosted registry and placement files under `.port/hosted/demo` can override static test bindings because registered nodes are resolved before static bindings. Isolated reruns then fail differently from full suites depending on leftover on-disk state. |
| **Suggested Action** | Clear default hosted runtime state in test helpers before starting a static-bound control plane, or use unique control-plane names per test when persisted state is part of the scenario. |
| **Applies To** | `crates/port-runtime/src/hosted_control_plane.rs`, hosted control-plane tests |
| **Linked Knowledge IDs** | |
| **Observed At** | 2026-03-09T00:40:00Z |
| **Score** | 0.86 |
| **Confidence** | 0.96 |
| **Applied** | yes |

## Observations

The durable registration path itself was stable once the targeted persistence tests were in place. The harder issue was test isolation: helper readiness probes and reused `demo` runtime files were leaking state into unrelated hosted proxy tests, which made isolated reruns fail even when the full suite passed.

Fixing the helper layer improved confidence in the product change. A dedicated mock readiness route stopped probes from polluting request assertions, node-agent readiness now waits for any HTTP response rather than only successful status, and the default static-binding control-plane helper now clears stale `demo` registry state before serving.

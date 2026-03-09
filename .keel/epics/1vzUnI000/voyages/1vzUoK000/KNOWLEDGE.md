---
created_at: 2026-03-09T01:37:45
---

# Knowledge - 1vzUoK000

> Automated synthesis of story reflections.

## Story Knowledge

## Story: Persist Hosted Registration And Freshness (1vzUq6000)

### 1vzVJp000: Isolate Default Hosted Test State From Demo Runtime Files

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | When hosted control-plane tests use the default `demo` control plane and static node bindings. |
| **Insight** | Persisted hosted registry and placement files under `.port/hosted/demo` can override static test bindings because registered nodes are resolved before static bindings. Isolated reruns then fail differently from full suites depending on leftover on-disk state. |
| **Suggested Action** | Clear default hosted runtime state in test helpers before starting a static-bound control plane, or use unique control-plane names per test when persisted state is part of the scenario. |
| **Applies To** | `crates/port-runtime/src/hosted_control_plane.rs`, hosted control-plane tests |
| **Applied** | yes |



---

## Synthesis

### kiRlqCTZ6: Isolate Default Hosted Test State From Demo Runtime Files

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | When hosted control-plane tests use the default `demo` control plane and static node bindings. |
| **Insight** | Persisted hosted registry and placement files under `.port/hosted/demo` can override static test bindings because registered nodes are resolved before static bindings. Isolated reruns then fail differently from full suites depending on leftover on-disk state. |
| **Suggested Action** | Clear default hosted runtime state in test helpers before starting a static-bound control plane, or use unique control-plane names per test when persisted state is part of the scenario. |
| **Applies To** | `crates/port-runtime/src/hosted_control_plane.rs`, hosted control-plane tests |
| **Linked Knowledge IDs** | 1vzVJp000 |
| **Score** | 0.86 |
| **Confidence** | 0.96 |
| **Applied** | yes |


---
created_at: 2026-03-08T19:10:55
---

# Reflection - Implement Hosted Streamed Forward Transport

## Knowledge

### 1vzMY2000: Hosted forward ownership can hide behind local listener setup
| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When a hosted guest capability has a working node-agent route but the CLI/runtime still reuses a local helper such as `prepare_guest_forward`. |
| **Insight** | The hosted control-plane path can be functionally live while the canonical CLI still bypasses it and silently falls back to local runtime assumptions. Forward ownership broke specifically because the CLI kept constructing a local session instead of entering the hosted `guest:forward` route. |
| **Suggested Action** | Add hosted-path tests that use a bogus client-side runtime root and require the control-plane/node-agent route to succeed or fail with hosted route context. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, `crates/port-cli/src/lib.rs`, hosted guest capability tests |
| **Linked Knowledge IDs** | |
| **Observed At** | 2026-03-08T19:11:00Z |
| **Score** | 0.86 |
| **Confidence** | 0.96 |
| **Applied** | yes |

## Observations

- The node-agent forward path already existed and was sufficient to prove the real hosted transport. The missing behavior was on the runtime/CLI side, which still rejected or bypassed hosted `Forward` requests before they reached that path.
- Using a bogus hosted client runtime root in the CLI tests was the right regression check. It proved the new behavior no longer depends on direct access to the node runtime socket layout.
- `cargo test` outside `nix develop` is not a trustworthy hygiene signal in this repo because the AVF tests expect shell-provided host tooling. The final verification pass needs to happen inside `nix develop`.

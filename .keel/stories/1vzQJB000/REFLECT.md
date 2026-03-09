---
created_at: 2026-03-08T19:46:33
---

# Reflection - Route Hosted Detached Forward Lifecycle

## Knowledge

### 1vzQLp000: Bogus Client Runtime Roots Are A Strong No-Fallback Hosted CLI Proof
| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | Verifying that hosted CLI commands no longer read repo-local runtime state after moving to the live control-plane and node-agent path |
| **Insight** | If the client config points the hosted node runtime root at a bogus path while the server-side config keeps the real runtime root, any successful hosted command proves the CLI is using remote transport rather than local state inspection. |
| **Suggested Action** | Keep using split server/client hosted configs with a bogus client runtime root in CLI integration tests for hosted transport stories. |
| **Applies To** | `crates/port-cli/tests/*`, hosted machine and guest transport tests |
| **Linked Knowledge IDs** | |
| **Observed At** | 2026-03-08T19:46:33Z |
| **Score** | 0.86 |
| **Confidence** | 0.97 |
| **Applied** | yes |

## Observations

This slice stayed small because the prior story already shipped the control
plane and node-agent runtime behavior. The CLI only needed to stop rejecting
hosted detached lifecycle flags, route those verbs to the new runtime helpers,
and print the same operator-facing fields the local lane already exposed.

The strongest regression proof was the split hosted config: the server uses the
real node runtime root, while the CLI client config points that runtime root at
a bogus path. Successful hosted `--list` and `--stop` commands under that setup
prove the CLI is no longer reading local runtime state on the client side.

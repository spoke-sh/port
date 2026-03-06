---
created_at: 2026-03-06T15:21:32
---

# Reflection - Deliver Guest Agent Capabilities

## Knowledge

### 1vyeM0000: Prefer In-Process Daemons For Workspace CLI Integration Tests
| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | When a CLI crate needs an integration test against a daemon implemented in another workspace crate |
| **Insight** | Spawning the daemon crate in-process through a dev-dependency is more reliable than discovering a sibling workspace binary from the test harness |
| **Suggested Action** | Prefer `thread::spawn` plus the daemon library entrypoint for workspace-local CLI integration tests unless the binary packaging itself is under test |
| **Applies To** | `crates/*/tests/*.rs`, workspace daemons, CLI integration tests |
| **Linked Knowledge IDs** | |
| **Observed At** | 2026-03-06T23:24:00Z |
| **Score** | 0.78 |
| **Confidence** | 0.90 |
| **Applied** | yes |

## Observations

- The shared protocol types and the host runtime client kept the CLI wiring small once the request and result envelopes were made explicit.
- The main implementation snag was test transport, not business logic: `cargo` integration tests in `port-cli` could not reliably discover the `port-guest-agent` binary from another workspace package.
- Updating `port --help` examples and the README at the same time kept the CLI surface aligned with the actual guest-agent behavior and its current socket-based limitation.

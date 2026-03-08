---
created_at: 2026-03-08T12:32:55
---

# Reflection - Implement Node Agent Pvm Launch Path

## Knowledge

### 1vzJSi000: Localize Hosted Node Launch Down To One Machine And Host
| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When a hosted node agent reuses the shared local launcher for one machine request |
| **Insight** | Rewriting only the host connection to `Local` is not enough. The localized config must be narrowed to the target machine and host, and hosted-only inventory must be removed, or config validation will fail on stale hosted references before the launch path runs. |
| **Suggested Action** | Keep node-agent launch localization as a deliberate scope-reduction step before calling shared local runtime helpers. |
| **Applies To** | `crates/port-runtime/src/hosted_control_plane.rs`, hosted-to-local runtime adaptation paths |
| **Linked Knowledge IDs** | 1vzJQg000 |
| **Observed At** | 2026-03-08T12:29:00Z |
| **Score** | 0.82 |
| **Confidence** | 0.95 |
| **Applied** | yes |

## Observations

The runtime slice stayed tractable once the failing proof moved to an HTTP node
agent test with a fake Firecracker binary. The main surprise was that the
shared local launcher still depended on standard-lane blocking doctor checks,
so PVM launch needed machine-specific preflight instead of reusing the full
repo-level doctor result verbatim.

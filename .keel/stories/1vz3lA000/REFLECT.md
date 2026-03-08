---
created_at: 2026-03-07T19:05:40
---

# Reflection - Plan Pvm Host Kit

## Knowledge

### 1vz4A6000: PVM Needs Host-Kit Contracts
| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Planning Firecracker/PVM support for cloud-cost-controlled Port execution |
| **Insight** | The PVM lane is not safely modeled as `protection_mode = "pvm"` on top of the standard Firecracker runtime. It needs an explicit host kit, artifact kit, and validation contract before runtime work is credible. |
| **Suggested Action** | When implementing PVM follow-on work, start with host-kit packaging and `port doctor` validation before wiring launch behavior. |
| **Applies To** | `crates/port-model/src/lib.rs`, `docs/pvm.md`, future `port doctor` and Firecracker/PVM driver work |
| **Linked Knowledge IDs** |  |
| **Observed At** | 2026-03-08T03:10:00Z |
| **Score** | 0.92 |
| **Confidence** | 0.97 |
| **Applied** | yes |

## Observations

- The upstream/vendor evidence was strong enough to keep x86_64 PVM in scope,
  but only under a much narrower contract than "turn on PVM in the existing
  driver."
- Encoding the decision in `port-model` made the docs sharper because the
  x86_64 keep versus arm64 research-only split could be tested instead of only
  described.
- The acceptance gate is much more reliable when story verification runs
  through repo-local scripts instead of nested shell commands embedded in the
  README.

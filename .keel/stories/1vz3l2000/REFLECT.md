---
created_at: 2026-03-07T19:10:53
---

# Reflection - Define AVF Execution Contract

## Knowledge

### 1vz4B5000: AVF Should Keep The Guest Protocol
| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Defining the first macOS AVF execution contract for Port |
| **Insight** | AVF does not require a second guest API. Port can keep the existing guest-agent model by mapping guest transport onto virtio sockets, console capture onto serial ports, and treating directory sharing as optional operator ergonomics rather than the control plane. |
| **Suggested Action** | Implement the AVF driver around virtio sockets plus serial ports first, then add directory sharing and Rosetta as explicit optional workflows. |
| **Applies To** | `crates/port-model/src/lib.rs`, `docs/avf.md`, future AVF driver and macOS `port doctor` work |
| **Linked Knowledge IDs** |  |
| **Observed At** | 2026-03-08T03:15:00Z |
| **Score** | 0.9 |
| **Confidence** | 0.95 |
| **Applied** | yes |

## Observations

- Apple's substrate primitives were a good fit for Port's current model once
  the design stopped assuming Firecracker-specific vsock and log paths.
- The sharpest macOS operator boundary was not launch syntax but packaging and
  entitlement expectations for distributed binaries.
- The same verify-script pattern used in the hosted and PVM stories worked well
  again here and avoided the `keel story record` quoting problems.

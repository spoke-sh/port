---
created_at: 2026-03-07T18:25:41
---

# Reflection - Extract Firecracker Driver Boundary

## Knowledge

### 1vz3uv000: Guest Forward Needs Endpoint-Level Driver Seams
| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Extracting substrate drivers for guest operations in `port-runtime` |
| **Insight** | A driver seam that only exposes `connect()` is not enough for long-lived flows like forwarding; the abstraction has to preserve a reusable guest-endpoint concept so each inbound connection can attach independently. |
| **Suggested Action** | When extracting additional substrate drivers, model guest attachment as endpoint resolution plus connection, not as a one-shot stream factory. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, future hosted and AVF guest transport work |
| **Linked Knowledge IDs** | |
| **Observed At** | 2026-03-08T02:25:00Z |
| **Score** | 0.77 |
| **Confidence** | 0.93 |
| **Applied** | yes |

## Observations

The refactor stayed small because the existing runtime already had a usable
seam: lifecycle functions and guest endpoint resolution were concentrated in one
module. That made it practical to introduce a real driver contract and a
concrete Firecracker implementation without destabilizing the current Linux
behavior.

The subtle part was guest forwarding. `exec` and `copy` can work with a single
connected stream, but `forward` needs a reusable endpoint so each inbound host
connection can attach separately. That is exactly the kind of detail the driver
boundary has to preserve if Port is going to add hosted and AVF lanes without
reworking guest semantics later.

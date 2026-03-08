---
created_at: 2026-03-07T18:25:41
---

# Reflection - Extract Firecracker Driver Boundary

## Knowledge

- [1vz3uv000](../../knowledge/1vz3uv000.md) Guest Forward Needs Endpoint-Level Driver Seams

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

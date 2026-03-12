# Cloud Block Storage Normalization — Brief

## Hypothesis

Port's cloud story will be much more credible if machines can request a
canonical block-storage contract across local, hosted, and provider-backed
execution instead of treating guest rootfs artifacts as the only meaningful
storage surface.

## Problem Space

Port already has a strong artifact and guest-image story, but it does not yet
publish a first-class portable block-storage contract for persistent or
ephemeral attached volumes. The user explicitly wants cloud block-storage
normalization, which means the current rootfs- and artifact-centric storage
model is not enough.

## Context

Today's repo describes:

- guest images and kernels as artifacts,
- hosted artifact storage under `.port/hosted/...`,
- and runtime launchers that pass a rootfs image to the hypervisor.

That is necessary, but it is not the same as normalized cloud block storage for
operator-facing workloads.

## Objectives

- Define the first canonical storage vocabulary for Port-managed machines.
- Distinguish rootfs artifacts from attachable or persistent block devices.
- Determine how storage contracts should map across local, hosted, and cloud
  execution lanes.
- Sequence the smallest storage slice that can later support higher-level
  service and Kubernetes use cases.

## Scope

- In scope: ephemeral versus persistent disk semantics, attachable block
  devices, snapshot or clone expectations, host-backed storage classes, and
  provider mapping.
- Out of scope: a full CSI implementation, distributed storage platform, or
  every possible filesystem backend.

## Success Criteria

- [ ] The research separates rootfs artifact handling from operator-visible
  block-storage contracts.
- [ ] One canonical storage vocabulary is concrete enough to plan across
  local, hosted, and cloud lanes.
- [ ] The first implementation slice is small enough to ship without pretending
  Port already has a full storage platform.
- [ ] Follow-on work for k3s and stateful workloads can point at this contract
  instead of inventing ad hoc disk semantics.

## Research Questions

- What should the first storage abstraction be: attached volumes, storage
  classes, or host-group storage capabilities?
- How much of the contract belongs in the shared model versus substrate-specific
  launchers?
- What snapshot, clone, or resize behavior is required in the first slice?

## Open Questions

- Should the first storage lane target only hosted cloud nodes, or also local
  workflows for parity?
- How should Port describe backend-specific implementations such as image files,
  ZFS, or devmapper without leaking those details into every workflow?

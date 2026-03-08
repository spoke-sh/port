# Cloud Substrate And PVM Strategy — Brief

## Hypothesis

Port needs a substrate-aware platform strategy rather than a Firecracker-only
host matrix. A credible next phase should preserve Firecracker and KVM as
important lanes, but add a serious plan for protected-VM execution on
cost-efficient cloud VMs, first-class hosted control-plane operation, and
additional substrates such as Apple Virtualization Framework on macOS. The
existing guest protocol and CLI can remain canonical, but they need a daemon/API
and richer machine lifecycle surfaces to span local and hosted operation.

## Problem Space

Port's completed MVP deliberately optimized for a narrow local Linux launch
path. That left major product gaps relative to SlicerVM:

- no hosted or remote control plane;
- no machine inventory, status, or stop lifecycle;
- no streamed PTY or log-follow semantics;
- no artifact push, pull, or remote cache story;
- no first-class multi-hypervisor design; and
- no supportable protected-VM lane for cloud cost control.

The user objective has changed. Port now needs a durable strategy for running on
cloud VMs even when nested virtualization is unavailable or too expensive, while
also supporting macOS operators and eventually a hosted Port environment. That
requires current research on Slicer's PVM claims, upstream protected
virtualization work, cloud-host cost boundaries, and the architectural changes
needed in Port's model, runtime, CLI, and artifact system.

## Success Criteria

- [x] The research distinguishes near-term, supportable execution lanes from aspirational ones across Firecracker KVM, Firecracker PVM, Cloud Hypervisor, and Apple Virtualization Framework.
- [x] The research clarifies what is true today about Slicer's PVM lane versus upstream arm64 protected-virtualization work.
- [x] The research yields a concrete recommendation for Port's control-plane, model, and artifact evolution, with at least one immediately plannable voyage.

## Open Questions

- Is Slicer's current PVM lane actually multi-architecture, or is the published support boundary still x86_64-only for that specific lane?
- Which protected-virtualization technologies are mature enough to matter for Port in the near term, and how do they map to Firecracker?
- What changes are required to carry Port's current guest transport and CLI model into a hosted control plane?
- How should Port represent substrates, protection modes, and artifact variants without creating a fragmented operator experience?
- Which gaps should land first to move Port toward Slicer-level capability without overcommitting to speculative platform work?

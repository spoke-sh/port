# PVM And Multi-Substrate Execution — Brief

## Hypothesis

Port cannot reach a credible Slicer-class roadmap by treating PVM, AVF, hosted
control, and cloud cost control as one generic "future lane". We likely need:

- an explicit keep/drop decision for each substrate and architecture claim,
- a substrate driver boundary in the runtime and CLI,
- a host-kit story for x86_64 PVM that includes kernel, VMM, and guest-image
  variants, and
- a separate first-class AVF lane for macOS rather than pretending Firecracker
  semantics extend there directly.

If that is true, the next planning unit should stop promising "planned" support
in the abstract and instead decompose PVM host enablement, AVF execution, and
hosted node-agent control as distinct workstreams.

## Problem Space

The current Port board is empty again, but the user's objective is not. Port is
still materially behind Slicer in three areas that are tightly coupled but not
identical:

- cloud cost control when nested virtualization or `/dev/kvm` is unavailable,
- first-class non-Firecracker substrates such as Apple Virtualization
  Framework, and
- a hosted daemon/control-plane architecture that can preserve today's CLI and
  guest protocol while moving lifecycle ownership away from the local process.

The immediate research question is which of those claims are technically real,
which are still research-only, and what concrete implementation slices follow
from that distinction.

## Success Criteria

How will we know if this research was valuable?

- [x] Determine whether Firecracker/PVM should be kept for Port on x86_64 and
  whether arm64 remains implementation scope or research-only.
- [x] Identify the concrete host, artifact, and runtime implications of a real
  PVM lane rather than a documentation-only lane.
- [x] Determine whether AVF deserves first-class planning as its own substrate
  lane and what that implies for guest transport and operator workflows.
- [x] Translate the findings into a concrete recommendation for the next epic
  or voyage instead of leaving the board starved.

## Open Questions

- Is x86_64 PVM mature enough to justify near-term Port investment, or is it
  still too custom to plan beyond research?
- Does any current evidence justify arm64 Firecracker/PVM as more than a
  research lane?
- How much of Actuated's arm64 story is PVM versus native arm hardware with
  normal virtualization support?
- What specific Port modules are still hard-wired to local Firecracker
  assumptions and therefore need substrate or hosted abstractions first?
- What is the smallest next planning slice that keeps Port honest while still
  moving toward Slicer-class capability?

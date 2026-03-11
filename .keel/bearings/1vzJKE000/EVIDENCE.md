---
id: 1vzJKE000
---

# Executable Pvm And Avf Lanes — Evidence

## Market Research

### Existing Solutions

Teams trying to control microVM cost in the cloud usually combine one of two
patterns:

- standard virtualization on hosts they control directly, often bare metal or
  nested-virt-capable infrastructure
- a managed platform that hides the substrate but does not expose a stable
  operator-facing guest workflow

That still leaves room for Port's approach: one canonical CLI and model across
local Linux, hosted Linux, and eventually macOS.

### Competitive Landscape

Slicer-like products are ahead on operational breadth, but their public arm64
story maps to native arm infrastructure and bare-metal execution, not proof
that arm64 Firecracker/PVM is already a commodity cloud-VM lane. That means
Port should separate "arm64 cost control is real" from "arm64 Firecracker/PVM
is executable now."

### Market Size

The opportunity is not every VM operator. It is teams that want:

- stronger isolation than containers
- one operator path across workstation and hosted execution
- cost-aware substrate choices instead of a single hypervisor assumption

## Technical Research

### Feasibility

Two executable programs are now technically justified:

1. x86_64 Firecracker/PVM on prepared Linux hosts.
2. Apple Virtualization Framework on macOS.

The supporting evidence is asymmetric:

- Firecracker upstream still presents a KVM-centered product. Port therefore
  needs its own host-kit contract for PVM rather than assuming an upstream
  ready-made lane.
- Existing LWN and lore references still support keeping x86_64 PVM while
  keeping arm64 Firecracker/PVM research-only.
- Apple exposes the AVF primitives Port needs for a real local macOS lane:
  Linux boot loaders, virtio sockets, and serial-port-backed console devices.

### Prior Art

Relevant prior-art patterns:

- Firecracker plus prepared Linux hosts for standard virtualization
- Actuated-style arm64 support on native arm infrastructure
- AVF-based Linux VMs on macOS using Apple's native virtualization APIs

Port can build on all three without collapsing them into one story.

### Proof of Concepts

Port already has the critical foundations needed for follow-on execution:

- a real local Firecracker launch path
- a real hosted control-plane and node-agent split
- shared guest-agent protocol semantics that can ride different transports

That means the next research step should feed planning, not another placeholder
design cycle.

## User Research

### Target Users

- Linux operators who need a credible protected lane for hosted fleets
- macOS operators who want a first-class local substrate instead of "SSH into a
  Linux box"
- teams that want arm64 cost control without false claims about arm64 PVM

### Pain Points

Today they still have to guess:

- which protected-execution promises are real versus research-only
- whether hosted readiness means real hosted launch
- how macOS fits into Port beyond documentation

### Validation

The user objective itself is the validation: Port is not done until it has a
real hosted Linux cost-control story and a first-class macOS story.

## Key Findings

1. Port should keep x86_64 Firecracker/PVM as the first protected Linux lane.
2. Port should keep arm64 Firecracker/PVM research-only and treat native arm64
   standard virtualization as the practical near-term arm64 cost-control path.
3. Port should split AVF into its own executable program instead of burying it
   under the Linux PVM queue.
4. The next Linux execution slice should focus on host-kit packaging plus
   prepared-node launch, because placement gating without launch is no longer
   enough.

## Unknowns

- What is the smallest portable packaging format for the PVM host kit across
  prepared Linux nodes?
- Whether Port should implement AVF directly in Rust bindings or via a narrow
  helper boundary.
- How much hosted control-plane state needs to become durable before prepared
  hosted PVM launch is worth productizing.

## Sources

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | web | manual:web-search | https://github.com/firecracker-microvm/firecracker | 2026-03-08 | 2026-03-08 | high | high | Firecracker upstream remains KVM-centered, supporting the need for Port-owned PVM host-kit work. |
| SRC-02 | web | manual:web-search | https://developer.apple.com/documentation/virtualization/ | 2026-03-08 | 2026-03-08 | high | high | Apple's Virtualization framework provides the primitives needed for a real AVF runtime lane. |
| SRC-03 | web | manual:web-search | https://docs.actuated.com/tasks/bring-your-own-cloud/ | 2026-03-08 | 2026-03-08 | medium | high | Actuated's documentation supports native-arm and host-controlled execution as distinct cost-control lanes. |
| SRC-04 | web | manual:web-search | https://docs.actuated.com/tasks/self-hosted-runners/ | 2026-03-08 | 2026-03-08 | medium | high | Hosted-runner docs reinforce the distinction between prepared-host execution and generic managed cloud VMs. |
| SRC-05 | manual | manual:doc-review | /home/alex/workspace/spoke-sh/port/docs/pvm.md | 2026-03-08 | 2026-03-08 | high | high | Port's PVM contract already defines host-kit readiness and launch constraints for the Linux lane. |
| SRC-06 | manual | manual:doc-review | /home/alex/workspace/spoke-sh/port/docs/avf.md | 2026-03-08 | 2026-03-08 | high | high | Port's AVF contract captures the macOS substrate expectations that need to become executable work. |

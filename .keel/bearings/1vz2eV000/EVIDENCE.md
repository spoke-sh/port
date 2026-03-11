---
id: 1vz2eV000
---

# Cloud Substrate And PVM Strategy — Evidence

## Market Research

### Existing Solutions

SlicerVM is now the clearest public comparison point for where Port needs to go.
Its public docs describe a broader product surface than Port's current MVP:

- a long-lived operational product with CLI, API, and SDK surfaces;
- multiple execution substrates, with Firecracker as the default Linux
  hypervisor, Cloud Hypervisor as an option, and Apple Virtualization Framework
  on macOS; and
- a documented protected-VM lane intended to run Firecracker on cloud VMs that
  do not expose `/dev/kvm` or nested virtualization.

Actuated strengthens the market signal that operators want microVM-backed
workloads across both x86_64 and arm64 infrastructure. But that does not by
itself prove that Slicer's current Firecracker PVM implementation is
multi-architecture. The current Slicer PVM documentation still scopes that
specific lane to `x86_64`.

### Competitive Landscape

The competitive gap has shifted since the original Port MVP:

- It is no longer enough to model remote Linux as a future lane with provider
  hints.
- Operators expect machine inventory, control-plane APIs, streaming guest
  access, artifact mobility, and host pooling.
- A strong cloud story now requires more than nested virtualization on premium
  instance types; it requires a position on protected virtualization and
  substrate selection.

### Market Signal

The commercial driver is cost control and host coverage. Teams want to run
microVM-backed workloads on regular cloud VMs where possible, reserve nested
virtualization and bare metal for when needed, and still keep one operator
surface across local, hosted, and hybrid environments.

## Technical Research

### Feasibility

The expansion is feasible, but only if Port splits it into explicit lanes rather
than promising one magic cloud substrate:

- Firecracker with normal KVM remains the proven baseline and should continue on
  both x86_64 and aarch64 where the host supports it.
- A Firecracker PVM lane is supportable as a dedicated engineering track, but
  current public evidence still points to a patch-heavy, specialized x86_64
  implementation rather than a turnkey multi-architecture solution.
- arm64 protected virtualization is technically real and increasingly upstream,
  but it should be treated as a parallel research/prototype lane until Port has
  a concrete Firecracker or alternate-hypervisor integration plan.
- A hosted control plane, artifact mobility, and richer machine lifecycle
  surfaces are feasible and should begin immediately because they are substrate-
  independent foundations.

### Slicer's Published PVM Boundary

Slicer's current public PVM docs state that the feature is:

- only supported with Firecracker;
- only supported on `x86_64`;
- dependent on custom host kernels, compatible guest images, and a custom
  Firecracker build; and
- configured explicitly through the machine setup stanza.

That means the user's instinct is directionally right but needs refinement:
Slicer clearly has an important PVM story, and the overall broader product
supports arm64 in other contexts, but the published support boundary for the
specific Firecracker PVM lane remains x86_64-only today.

### Upstream Protected Virtualization

Upstream protected virtualization is broader than Slicer's current PVM docs.
Current sources point to two important realities:

1. arm64 protected KVM (`pKVM`) is real, upstream, and actively advancing.
2. Slicer-style Firecracker PVM on cloud VMs is still patch-heavy and not a
   drop-in equivalent to generic upstream pKVM support.

The kernel and LWN material shows protected virtualization maturing on arm64.
Protected KVM has been under active upstream development for several kernel
cycles, and recent LWN coverage indicates more pKVM support landing in Linux
6.15. Android's pKVM documentation also shows a concrete arm64 implementation
model with guest hypercalls and host/guest coordination semantics.

The distinct Alibaba/Ant PVM framework discussed in 2024 lore and LWN material
is a different signal: it shows serious momentum for running KVM guests in a
more protected mode on conventional cloud VMs, but it also underscores that the
framework is still RFC-grade, large, and operationally specialized.

### Slicer's PVM Implementation Reality

Alex Ellis' 2025 write-up is especially useful because it collapses the product
claim into operational reality. His account describes:

- patching the host kernel to enable the PVM framework;
- patching Firecracker itself;
- adapting guest images;
- significant custom work for container workloads; and
- a conclusion that the current state is best suited to early adopters targeting
  specific clouds and instance types.

That is the right operational reading for Port as well. PVM is strategically
important, but it is not cheap scope. A serious Port PVM lane would need its own
host-kernel pipeline, Firecracker distribution path, artifact variants, and
provider validation matrix.

### Implications For Arm64

Arm64 matters, but we need to separate three claims:

1. Port should remain multi-architecture overall.
2. Upstream arm64 protected virtualization is real.
3. Slicer's currently documented Firecracker PVM lane is still x86_64-only.

Those can all be true at the same time. For Port, that means:

- keep aarch64 as a first-class architecture for normal Firecracker/KVM and
  Apple Virtualization Framework lanes;
- treat arm64 protected virtualization as an active research and design lane;
  and
- do not assume that upstream arm64 pKVM automatically yields a production-ready
  Firecracker PVM story without additional prototype work.

### Hosted Control Plane Implications

Port's current implementation has a strong local primitive: the guest protocol
already works against both a host-local Unix socket and a live Firecracker vsock
tunnel. That is a good basis for a hosted product because it suggests one
canonical guest operation model with multiple transport backends.

What is missing is the hosted control plane around it:

- a long-lived daemon or service to own machine lifecycle;
- a remote API and SDK surface;
- machine inventory, status, and stop semantics;
- transport brokering from remote clients to local guest channels; and
- authentication, authorization, and host-group scheduling.

### Artifact-System Implications

Port's local artifact build and validation surface is good, but it stops at the
host filesystem. A hosted and multi-substrate product needs:

- architecture- and substrate-aware artifact metadata;
- build, push, pull, and cache semantics;
- provider-friendly remote distribution;
- variant selection for normal KVM versus protected lanes; and
- clear contracts for kernels, guest images, and any substrate-specific runtime
  shims.

### Multi-Hypervisor Implications

Port should no longer treat "provider" as the main axis of runtime capability.
The more important axis is substrate:

- Firecracker on Linux with KVM;
- Firecracker on Linux with protected virtualization where supportable;
- Cloud Hypervisor on Linux for workloads that benefit from that lane; and
- Apple Virtualization Framework on macOS as a first-class local operator lane.

Provider identity still matters, but it should become one part of a broader
capability model rather than the primary design dimension.

## User Research

### Target Users

The target users are no longer just local Linux operators experimenting with
Firecracker. The practical user set now includes:

- platform teams running hosted or hybrid Port deployments;
- cloud cost-sensitive teams avoiding bare metal and high-end nested-virt
  families where possible;
- macOS operators who need a first-class local story; and
- teams that want one CLI and API model for local, hosted, and managed
  execution.

### Pain Points

Compared with Slicer, Port currently leaves these user needs unmet:

- lifecycle visibility and control for running machines;
- durable remote operations instead of one-shot local process launch;
- live shell and log streaming semantics;
- artifact mobility and caching;
- protected-VM and alternate-hypervisor choices; and
- a clear hosted product story.

### Validation

Validation for this research is strong if it yields an execution plan that can
start immediately with coherent stories instead of broad aspirations.

## Key Findings

1. Slicer's published PVM lane is strategically important but still specialized:
   x86_64-only, Firecracker-only, and dependent on custom kernels, guest-image
   compatibility, and patched Firecracker.
2. Upstream protected virtualization, especially arm64 pKVM, is real and moving
   forward, but it is not the same thing as a ready-made Firecracker PVM lane.
3. Port should remain multi-architecture overall, but must not conflate "arm64
   exists somewhere in the ecosystem" with "our Firecracker PVM lane is ready on
   arm64."
4. The right architectural pivot for Port is from a provider-only cloud matrix
   toward a substrate-aware control plane with provider, hypervisor, protection
   mode, transport, and artifact-variant modeling.
5. The nearest practical path toward Slicer-like breadth starts with hosted
   control-plane foundations and richer machine lifecycle surfaces, not with
   trying to ship every substrate in one slice.

## Unknowns

- Which protected-virtualization technology is the best long-term fit for an
  arm64 Port lane.
- How much of Slicer's PVM implementation is portable across cloud providers
  without provider-specific kernel packaging.
- Whether Cloud Hypervisor should be a first-class operator choice or mainly a
  specialized Linux lane.
- What the cleanest artifact registry and distribution contract should be for
  hosted Port.

## Sources

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | web | manual:web-search | https://docs.slicervm.com/ | 2026-03-07 | 2026-03-07 | medium | high | Slicer documents the broader hosted CLI/API/SDK and multi-substrate product surface Port is comparing against. |
| SRC-02 | web | manual:web-search | https://docs.slicervm.com/tasks/pvm/ | 2026-03-07 | 2026-03-07 | medium | high | Slicer scopes its current Firecracker PVM lane to custom components and `x86_64`. |
| SRC-03 | web | manual:web-search | https://docs.slicervm.com/reference/api/ | 2026-03-07 | 2026-03-07 | medium | high | Slicer's product surface includes a real API, supporting the hosted-control-plane implications in this bearing. |
| SRC-04 | web | manual:web-search | https://blog.alexellis.io/how-to-run-firecracker-without-kvm-on-regular-cloud-vms/ | 2026-03-07 | 2026-03-07 | medium | medium | Operational write-up showing current Firecracker PVM lanes remain patch-heavy and specialized. |
| SRC-05 | web | manual:web-search | https://lwn.net/Articles/848284/ | 2026-03-07 | 2026-03-07 | high | medium | LWN's protected-KVM coverage supports the claim that upstream arm64 protected virtualization is real. |
| SRC-06 | web | manual:web-search | https://lwn.net/Articles/1055029/ | 2026-03-07 | 2026-03-07 | high | high | Recent LWN coverage supports continued movement in pKVM and protected virtualization upstream. |
| SRC-07 | web | manual:web-search | https://lore.kernel.org/lkml/CABgObfaSGOt4AKRF5WEJt2fGMj_hLXd7J2x2etce2ymvT4HkpA@mail.gmail.com/T/ | 2026-03-07 | 2026-03-07 | medium | medium | Lore thread shows the PVM framework as active but still RFC-grade and operationally specialized. |
| SRC-08 | web | manual:web-search | https://source.android.com/docs/core/virtualization/pkvm-hypercalls | 2026-03-07 | 2026-03-07 | high | high | Android's pKVM documentation provides a concrete arm64 protected-virtualization implementation model. |

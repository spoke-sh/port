---
id: 1vz3ck000
---

# PVM And Multi-Substrate Execution — Evidence

## Sources

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | web | manual:web-search | https://lwn.net/Articles/963718/ | 2026-03-07 | 2026-03-07 | high | medium | LWN describes the x86 pagetable-based PVM work and supports the host-kit-heavy interpretation of PVM. |
| SRC-02 | web | manual:web-search | https://lkml.rescloud.iu.edu/2402.3/04599.html | 2026-03-07 | 2026-03-07 | medium | medium | The RFC thread supports the claim that the x86 PVM lane remained specialized and nontrivial. |
| SRC-03 | web | manual:web-search | https://docs.slicervm.com/tasks/pvm/ | 2026-03-07 | 2026-03-07 | medium | high | Slicer documents a current Firecracker PVM lane on `x86_64`, supporting the keep/drop decision. |
| SRC-04 | web | manual:web-search | https://docs.slicervm.com/getting-started/slicer-for-mac/ | 2026-03-07 | 2026-03-07 | medium | high | Slicer for Mac supports treating AVF as its own first-class substrate lane. |
| SRC-05 | web | manual:web-search | https://developer.apple.com/documentation/virtualization/running-intel-binaries-in-linux-vms-with-rosetta | 2026-03-07 | 2026-03-07 | high | high | Apple's AVF documentation supports the feasibility of a real macOS virtualization lane. |
| SRC-06 | web | manual:web-search | https://docs.actuated.com/tasks/faq/ | 2026-03-07 | 2026-03-07 | medium | medium | Actuated's FAQ supports separating native arm hardware lanes from PVM claims. |
| SRC-07 | manual | manual:code-inspection | /home/alex/workspace/spoke-sh/port/crates/port-runtime/src/lib.rs | 2026-03-07 | 2026-03-07 | high | high | Local runtime inspection supports the finding that Port still couples lifecycle ownership to Firecracker-local code paths. |

## Findings

### 1. x86_64 Firecracker/PVM is real, but it is a host-kit problem, not a config flag

The LWN coverage of the x86 pagetable-based PVM RFC describes a design that
protects guest memory from the host VMM by switching to shadow paging and
adding new KVM/MM behavior. The reported tradeoff is explicit: it avoids
hardware nested-virtualization requirements, but it carries performance and
feature constraints and depends on KVM/MM changes that were still RFC-quality
in early 2024.

Slicer's public PVM docs line up with that reading:

- PVM requires a custom host kernel image.
- PVM requires a dedicated Firecracker build or release.
- PVM requires compatible Slicer images.
- PVM is currently limited to Firecracker.
- PVM is currently limited to `x86_64`.

Implication for Port:
shipping PVM means Port must own or integrate a host kit:

- kernel build/distribution,
- Firecracker build/distribution,
- artifact variant generation for the PVM lane,
- and validation/reporting that distinguishes "host prepared for PVM" from
  "standard KVM host".

This is substantially more than extending the current machine model.

### 2. arm64 evidence does not currently justify a Firecracker/PVM implementation claim

Port's current docs already treat Firecracker/PVM on `aarch64` as research
only, and the external evidence supports keeping it there:

- Slicer's own PVM docs state that the current PVM support is `x86_64` only.
- The LWN / LKML material reviewed here is about x86 pagetable-based PVM, not a
  shipped arm64 Firecracker/PVM runtime.
- No current upstream or product source reviewed in this bearing established a
  supportable arm64 Firecracker/PVM path that Port could honestly claim.

The user's observation about Actuated and arm64 is directionally useful, but it
does not overturn the conclusion above. The evidence reviewed here indicates
that Actuated's arm64 capability is about running on native arm hardware, not
about proving an arm64 Firecracker/PVM lane on generic cloud VMs without
nested virtualization.

### 3. Actuated's arm64 story is native hardware / bare metal, not proof of arm64 PVM

Actuated's FAQ emphasizes bare-metal execution as the baseline and explicitly
calls out that nested virtualization is not supported in their service. Their
documented ARM guidance points to arm64 bare-metal runners. That is important,
but it means something different from "arm64 PVM is solved":

- native arm hosts are a valid cost/performance lane,
- they do not remove the need for a real PVM lane on commodity x86 cloud VMs,
- and they do not justify claiming arm64 Firecracker/PVM support in Port.

Implication for Port:
we should distinguish three cloud cost-control strategies rather than merging
them:

- standard KVM on native-capable hosts,
- x86_64 Firecracker/PVM on prepared hosts,
- and native arm execution on real arm hardware.

### 4. AVF is a first-class substrate lane, not an extension of the Firecracker runtime

Slicer for Mac documents a real macOS operator product built on Apple's
Virtualization Framework, with a different operational shape than Linux
Firecracker. Apple also documents Linux guest features such as Rosetta support.

This suggests AVF should stay in scope for Port, but as its own driver lane:

- different launch/runtime APIs,
- different transport and file-sharing primitives,
- different operator expectations on macOS,
- and likely a different node-agent implementation in the hosted product.

Port can preserve the CLI verbs and guest protocol semantics, but it should not
pretend that the current Firecracker runtime module can simply grow an `avf`
flag and stay coherent.

### 5. Port's current runtime still assumes local Firecracker ownership in the critical path

Local code inspection shows that Port has useful reusable seams, but the main
runtime still bakes in Firecracker/local assumptions:

- [crates/port-runtime/src/lib.rs](../../../crates/port-runtime/src/lib.rs)
  owns launch, runtime-root inspection, Firecracker config generation, process
  management, and guest-vsock tunneling in one module.
- [crates/port-runtime/src/lib.rs](../../../crates/port-runtime/src/lib.rs)
  launches Firecracker directly with `--no-api` and local files plus PIDs as
  the lifecycle source of truth.
- [crates/port-runtime/src/lib.rs](../../../crates/port-runtime/src/lib.rs)
  assumes the guest transport is either a host-local Unix socket or a
  Firecracker-vsock bridge.
- [crates/port-cli/src/lib.rs](../../../crates/port-cli/src/lib.rs)
  still renders a mostly local/runtime-root lifecycle model.
- [crates/port-model/src/lib.rs](../../../crates/port-model/src/lib.rs)
  now carries substrate and artifact vocabulary, but it still does not define a
  substrate driver contract or hosted transport contract at the type boundary.

The good news is that the guest protocol itself is reusable. The hosted docs
and live guest transport work already show the right seam:

- keep CLI verbs canonical,
- keep guest protocol canonical,
- move launch, inventory, and transport attachment behind node-agent/substrate
  drivers.

### 6. The next planning slice should be substrate drivers plus host kits, not more documentation-only lanes

The board should not add another "planned support matrix" voyage. The research
points toward a concrete decomposition:

1. substrate driver boundary for launch/status/stop/guest attach,
2. hosted node-agent runtime ownership over that boundary,
3. x86_64 PVM host-kit and artifact-kit lane,
4. AVF macOS driver lane,
5. productized lifecycle and inventory surfaces that can target local or hosted
   backends.

Without that cut, Port will keep accumulating modeled lanes that cannot be
implemented without ripping through the Firecracker-local runtime later.

## Open Technical Risks

- x86_64 PVM remains custom enough that Port may need to own packaging and
  validation for patched host components.
- The current runtime couples transport, launch, and status tightly enough that
  a substrate abstraction could trigger a larger refactor than a small feature
  story.
- AVF can become another docs-only lane if we do not immediately create a real
  driver and operator workflow plan.

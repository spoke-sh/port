---
id: VDcStPNlr
---

# GPU Execution Support — Evidence

## Sources

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | manual | manual:repo-inspection | /home/alex/workspace/spoke-sh/port | 2026-03-11 | 2026-03-11 | high | high | Current repo inspection shows no first-class GPU or accelerator contract in docs, model, or runtime help surfaces. |
| SRC-02 | web | manual:web-open | https://docs.slicervm.com/examples/gpu-ollama/ | 2026-03-11 | 2026-03-11 | medium | high | Slicer documents a GPU passthrough workflow that starts with explicit `gpu_count`, Cloud Hypervisor, and x86_64 or VFIO requirements. |
| SRC-03 | web | manual:web-open | https://docs.slicervm.com/examples/k3s-gpu/ | 2026-03-11 | 2026-03-11 | medium | high | A GPU-backed k3s example shows that accelerators become much more compelling when paired with a higher-level workload story. |
| SRC-04 | manual | manual:doc-review | /home/alex/workspace/spoke-sh/port/docs/cloud.md | 2026-03-11 | 2026-03-11 | high | high | `docs/cloud.md` already publishes multiple substrate lanes and hosted placement patterns that a future GPU contract must fit into. |

## Feasibility

Feasible, but as a narrow substrate and placement contract rather than a broad
claim. The current repo has no accelerator surface yet, so the right first step
is explicit capability modeling and one credible proof lane.

## Findings

### 1. GPU support is currently greenfield for Port

Repo inspection shows no current GPU or accelerator contract in the shared
docs, model, or runtime help surfaces. That means the board needs a dedicated
research package before implementation starts [SRC-01].

### 2. External precedent favors a narrow passthrough-first lane

Slicer's GPU examples tie accelerator support to explicit host-group settings,
Cloud Hypervisor, VFIO, and an x86_64 platform boundary. That suggests Port
should begin with a similarly explicit and narrow lane rather than claiming
generic accelerator support [SRC-02][SRC-03].

### 3. GPU support intersects heavily with placement and workload orchestration

The most interesting external GPU examples are not just "boot one VM." They
pair accelerators with recognizable workloads such as Ollama or k3s. That means
Port's GPU work should likely align with hosted placement and possibly k3s
planning instead of standing alone as a low-level device feature [SRC-03][SRC-04].

## Open Technical Risks

- Hardware-specific constraints can make the first GPU lane fragile if vendor,
  kernel, IOMMU, and substrate requirements are not stated explicitly.
- A substrate choice that works for GPUs may diverge from the default
  Firecracker lane and require sharper operator messaging.
- Without a strong proof artifact, the work could feel impressive
  technically but still low-signal for humans evaluating the product.

## Key Findings

1. Port needs explicit research because no first-class GPU contract exists yet
   [SRC-01].
2. A passthrough-first lane with explicit host capability boundaries is the
   most credible starting point [SRC-02].
3. GPU work is most valuable when paired with hosted placement and higher-level
   workload stories such as k3s [SRC-03][SRC-04].

## Unknowns

- Which substrate should own the first GPU lane in Port?
- Is the first proof better as a single accelerated VM, a service workflow, or
  a GPU-backed k3s cluster?

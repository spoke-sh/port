---
id: VDcStQqlo
---

# Cloud Block Storage Normalization — Evidence

## Sources

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | manual | manual:doc-review | /home/alex/workspace/spoke-sh/port/docs/artifacts.md | 2026-03-11 | 2026-03-11 | high | high | The current storage vocabulary is centered on artifacts, caches, hosted artifact paths, and guest-image variants. |
| SRC-02 | manual | manual:code-inspection | /home/alex/workspace/spoke-sh/port/crates/port-runtime/src/lib.rs | 2026-03-11 | 2026-03-11 | high | high | Current launch paths pass one rootfs disk artifact to the hypervisor, which is necessary but not a full block-storage contract. |
| SRC-03 | web | manual:web-open | https://docs.slicervm.com/storage/overview/ | 2026-03-11 | 2026-03-11 | medium | high | Slicer documents explicit storage modes such as disk images and CoW-backed snapshotting. |
| SRC-04 | web | manual:web-open | https://docs.slicervm.com/storage/devmapper/ | 2026-03-11 | 2026-03-11 | medium | high | Devmapper configuration shows one concrete example of explicit host-group storage policy instead of implicit disk behavior. |

## Feasibility

Feasible, but only if Port treats storage as a first-class machine or host
capability contract instead of trying to stretch the current artifact layout
into every persistence problem.

## Findings

### 1. Port's current storage story is artifact-first

`docs/artifacts.md` makes it clear that Port already handles kernels, guest
images, cache roots, hosted artifact stores, and distribution backends well.
Those are necessary storage primitives, but they are still artifact movement
and rootfs selection, not a normalized block-storage product contract [SRC-01].

### 2. Runtime launchers currently consume a rootfs image, not a volume model

The current runtime code passes a rootfs image path and read-only flag into the
hypervisor launch path. That means Port has a strong execution primitive, but
not yet a user-facing persistent-volume or attached-disk abstraction [SRC-02].

### 3. External precedent keeps storage backends explicit

Slicer documents storage explicitly in terms of image, CoW-backed storage, and
devmapper-backed behavior. That is a useful product lesson: Port should publish
storage classes and lifecycle semantics intentionally instead of hiding backend
choices behind vague "cloud storage" language [SRC-03][SRC-04].

## Open Technical Risks

- Storage abstractions can leak substrate or provider detail too aggressively if
  they are not kept at the right layer in the shared model.
- Persistent volume semantics will interact with hosted placement, migration,
  and failure recovery much earlier than simple rootfs artifacts do.
- The first slice could sprawl into a full storage platform if snapshot,
  resize, clone, and replication are all attempted at once.

## Key Findings

1. Port currently has an artifact and rootfs story, not a normalized
   block-storage contract [SRC-01][SRC-02].
2. A credible cloud-storage lane needs explicit backend and lifecycle
   vocabulary [SRC-03][SRC-04].
3. Storage normalization should sit above substrate-specific launchers but
   below higher-level service and cluster workflows [SRC-02][SRC-03].

## Unknowns

- What is the smallest useful first slice: one attached writable volume, or a
  small storage-class model?
- How should persistent storage interact with hosted node placement and future
  k3s orchestration?

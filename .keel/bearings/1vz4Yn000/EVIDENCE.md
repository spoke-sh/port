---
id: 1vz4Yn000
---

# Hosted Control Plane And Operator Surface — Evidence

## Sources

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | web | manual:web-search | https://docs.slicervm.com/ | 2026-03-07 | 2026-03-07 | medium | high | Slicer docs show the broader hosted product surface Port is trying to sequence. |
| SRC-02 | web | manual:web-search | https://docs.slicervm.com/reference/api/ | 2026-03-07 | 2026-03-07 | medium | high | Slicer's API reference supports the need for an authenticated remote lifecycle surface. |
| SRC-03 | manual | manual:doc-review | /home/alex/workspace/spoke-sh/port/docs/hosted.md | 2026-03-07 | 2026-03-07 | high | high | Port's hosted docs capture intent but not yet a productized remote control surface. |
| SRC-04 | manual | manual:code-inspection | /home/alex/workspace/spoke-sh/port/crates/port-runtime/src/lib.rs | 2026-03-07 | 2026-03-07 | high | high | Local runtime inspection supports the finding that lifecycle and guest brokerage remain local-process owned. |

## Feasibility

Feasible, but only if Port sequences the next work as a hosted-control
foundation rather than a grab bag of individual operator features.

## Findings

### 1. Port's next gap is hosted product surface, not another substrate-only contract

Port now has explicit contracts for hosted ownership, x86_64 PVM, and AVF, but
those slices still stop short of a real hosted product surface. The biggest
remaining gap versus the user's objective and Slicer is the absence of:

- authenticated remote lifecycle control,
- node-aware inventory,
- a real hosted API surface,
- and operator visibility beyond local runtime roots.

This is now the critical-path gap because the substrate contracts already
define where Port wants to go. The missing step is turning that design into a
product surface.

### 2. Slicer's product breadth clusters around one control-plane foundation

Slicer's public docs describe more than isolated feature points. The features
cluster around a shared hosted-control foundation:

- authenticated API and CLI,
- node or host-group placement,
- remote machine lifecycle,
- guest-operation brokerage,
- monitoring,
- and higher-level workflow primitives.

That implies Port should not copy isolated verbs one at a time. The next Port
epic should build the hosted-control foundation that these operator surfaces
share.

### 3. Auth, inventory, and node ownership are the common prerequisites

Most of the user-requested missing features depend on the same underlying
capabilities:

- auth or API identity,
- control-plane inventory,
- node or host-group vocabulary,
- and guest-bridge attachment through the node owner.

Without those pieces, local-only ergonomics such as `top`, detached forwards,
or service vocabularies risk becoming another round of local-only surface area
that must later be redesigned.

### 4. The best next slice is hosted `machine list|status|stop` over an authenticated API

Port already has the right CLI vocabulary. The lowest-risk, highest-leverage
way to make the hosted story real is to keep those verbs and back them with:

- token-based auth,
- a control-plane API skeleton,
- node or host-group selection terms,
- and remote inventory or lifecycle queries.

That turns hosted Port from "documented future" into a working control surface
without having to ship secrets, services, sandboxes, and SDK packaging all at
once.

### 5. Secrets, monitoring, services, sandboxes, and SDK should follow the same foundation

These features are still required for the broader user objective, but the
survey indicates they should follow, not precede, the first hosted-control
foundation:

- monitoring or `top`
- secrets
- services and sandboxes
- detached forwards and Unix-socket forwarding
- SDK packaging

They become much easier to sequence once Port has stable auth, API identity,
node ownership, and hosted lifecycle semantics.

## Open Technical Risks

- A hosted API can sprawl if node, host-group, lifecycle, and guest-bridge
  concepts are not defined tightly in the first voyage.
- Pulling too many downstream features into the next epic would recreate the
  same "broad wishlist" problem instead of giving Port an executable control
  surface.
- A hosted daemon without a clean API/auth contract would risk diverging from
  the current CLI model and guest protocol semantics.

## Key Findings

1. Hosted control is now the next highest-leverage Port gap.
2. Auth, inventory, and node ownership are the shared prerequisites for most
   remaining Slicer-class product features.
3. The next epic should start with hosted `machine list|status|stop` and the
   hosted guest-connect primitive rather than downstream ergonomics.

## Unknowns

- How much SDK work should land in the same epic versus after the API settles?
- Whether the first hosted slice should include monitoring or leave it for the
  immediately following voyage.

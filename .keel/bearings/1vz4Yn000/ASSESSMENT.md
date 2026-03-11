---
id: 1vz4Yn000
---

# Hosted Control Plane And Operator Surface — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 5 | This is the clearest path from Port's current local-plus-contract state to the hosted product surface the user asked for. |
| Confidence | 4 | The current Port docs and Slicer public docs line up strongly on what the next shared foundation needs to be. |
| Effort | 5 | Auth, API identity, node inventory, and hosted lifecycle control are cross-cutting product work. |
| Risk | 4 | The main risk is trying to land too many downstream operator features before the hosted-control foundation is stable. |

*Scores range from 1-5:*
- 1 = Very Low
- 2 = Low
- 3 = Medium
- 4 = High
- 5 = Very High

## Analysis

### Findings

- Hosted lifecycle control, API identity, and operator visibility are the next
  highest-leverage gaps between Port's current state and a real hosted product
  surface [SRC-01][SRC-03][SRC-04].
- Auth, inventory, node ownership, and guest brokerage form the shared
  foundation most downstream operator features depend on [SRC-02][SRC-04].

### Opportunity Cost

The opportunity cost is delaying some user-visible operator ergonomics such as
`top`, secrets, or service or sandbox workflows. That delay is justified
because those features all become more coherent once Port has a real hosted
control plane instead of a design-only hosted contract [SRC-01][SRC-03][SRC-04].

### Dependencies

The next epic depends on:

- one authenticated API identity model [SRC-02][SRC-04],
- a node or host-group vocabulary [SRC-01][SRC-02],
- hosted machine inventory and lifecycle contracts [SRC-01][SRC-03][SRC-04],
- and a guest-connect or bridge primitive that preserves the existing guest
  protocol [SRC-03][SRC-04].

### Alternatives Considered

Alternatives considered:

- Expand more local-only operator verbs first:
  rejected because it would keep Port behind on the hosted product axis the
  user explicitly prioritized [SRC-01][SRC-03].
- Implement a node agent before the authenticated API and inventory model:
  rejected because it risks inventing daemon semantics that later diverge from
  the control plane [SRC-02][SRC-04].
- Jump directly to secrets, services, or sandboxes:
  rejected because those features depend on the same hosted-control foundation
  and would arrive on unstable ground [SRC-01][SRC-02][SRC-04].

## Recommendation

- [x] Proceed → create a hosted-control expansion epic immediately [SRC-01][SRC-02][SRC-03][SRC-04]
- [ ] Park → revisit later [SRC-03]
- [ ] Decline → document learnings [SRC-04]

Proceed with a hosted-control expansion epic whose first voyage establishes:

1. auth token and API identity,
2. node or host-group vocabulary,
3. hosted `machine list|status|stop`,
4. and the first guest-connect brokerage primitive.

Then layer monitoring, secrets, services, sandboxes, detached forwards,
Unix-socket forwarding, and SDK work on top of that foundation instead of
trying to land all of them in the same first slice [SRC-01][SRC-02][SRC-03][SRC-04].

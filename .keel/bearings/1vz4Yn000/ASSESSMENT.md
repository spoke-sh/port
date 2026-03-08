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

### Opportunity Cost

The opportunity cost is delaying some user-visible operator ergonomics such as
`top`, secrets, or service or sandbox workflows. That delay is justified
because those features all become more coherent once Port has a real hosted
control plane instead of a design-only hosted contract.

### Dependencies

The next epic depends on:

- one authenticated API identity model,
- a node or host-group vocabulary,
- hosted machine inventory and lifecycle contracts,
- and a guest-connect or bridge primitive that preserves the existing guest
  protocol.

### Alternatives Considered

Alternatives considered:

- Expand more local-only operator verbs first:
  rejected because it would keep Port behind on the hosted product axis the
  user explicitly prioritized.
- Implement a node agent before the authenticated API and inventory model:
  rejected because it risks inventing daemon semantics that later diverge from
  the control plane.
- Jump directly to secrets, services, or sandboxes:
  rejected because those features depend on the same hosted-control foundation
  and would arrive on unstable ground.

## Recommendation

- [x] Proceed → create a hosted-control expansion epic immediately
- [ ] Park → revisit later
- [ ] Decline → document learnings

Proceed with a hosted-control expansion epic whose first voyage establishes:

1. auth token and API identity,
2. node or host-group vocabulary,
3. hosted `machine list|status|stop`,
4. and the first guest-connect brokerage primitive.

Then layer monitoring, secrets, services, sandboxes, detached forwards,
Unix-socket forwarding, and SDK work on top of that foundation instead of
trying to land all of them in the same first slice.

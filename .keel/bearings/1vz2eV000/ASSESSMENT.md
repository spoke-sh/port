---
id: 1vz2eV000
---

# Cloud Substrate And PVM Strategy — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 5 | This work determines whether Port can become a credible hosted and cloud-cost-effective product rather than remaining a local Firecracker tool. |
| Confidence | 4 | The public product and upstream signals are strong enough to guide architecture, even though the eventual PVM implementation will require prototype validation. |
| Effort | 5 | Hosted control, multi-substrate support, and a serious PVM lane span model, runtime, CLI, artifacts, docs, and operations. |
| Risk | 4 | The main risk is treating several distinct technologies as one lane and overcommitting to unsupported platform promises. |

*Scores range from 1-5:*
- 1 = Very Low
- 2 = Low
- 3 = Medium
- 4 = High
- 5 = Very High

## Analysis

### Opportunity Cost

The opportunity cost is substantial: broadening Port this way delays polish on
the original narrow local tool. But the user objective has explicitly changed,
and the competitive comparison makes the old scope insufficient. Avoiding the
work would leave Port structurally behind on the exact axes that matter for a
hosted product.

### Dependencies

Port now depends on a sequence of architectural shifts:

- a new substrate-aware model beyond the current provider-only framing;
- a hosted control plane with API/SDK and machine lifecycle ownership;
- richer runtime manifests and inventory/status semantics;
- artifact distribution contracts for local and remote use; and
- dedicated research and prototype slices for protected virtualization and Apple
  Virtualization Framework.

### Alternatives Considered

Alternatives considered:

- Keep PVM scoped out again:
  rejected because cloud cost control is now a primary product objective.
- Treat arm64 as proof that Firecracker PVM is already solved:
  rejected because current public evidence does not support that conclusion.
- Focus only on control-plane ergonomics and ignore substrate work:
  rejected because hosted value depends on having cost-effective execution lanes.
- Focus only on PVM and defer lifecycle/API expansion:
  rejected because Port still needs the control-plane scaffolding that would
  make any new substrate usable as a product.

## Recommendation

- [x] Proceed → create a new expansion epic and begin implementation from hosted-control and lifecycle foundations
- [ ] Park → revisit later
- [ ] Decline → document learnings

Proceed with a multi-lane strategy:

1. Keep Firecracker with normal KVM as the primary proven lane.
2. Reopen PVM as an explicit product lane, starting with a dedicated Firecracker
   PVM track and treating arm64 protected virtualization as adjacent but not
   assumed solved.
3. Introduce substrate-aware planning for Cloud Hypervisor and Apple
   Virtualization Framework.
4. Start implementation from the hosted control-plane foundation, machine
   lifecycle surfaces, and artifact mobility, because every later substrate will
   need those product surfaces anyway.

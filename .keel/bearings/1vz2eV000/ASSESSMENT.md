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

### Findings

- Firecracker with normal KVM remains the baseline lane, while Firecracker PVM
  remains strategically important but specialized and operationally heavy
  [SRC-02][SRC-04].
- Upstream protected virtualization on arm64 is real, but it does not amount to
  a ready-made Firecracker PVM claim for Port [SRC-05][SRC-06][SRC-08].
- Hosted lifecycle ownership and artifact mobility are substrate-independent
  foundations Port needs regardless of which execution lanes it adds
  [SRC-01][SRC-03].

### Opportunity Cost

The opportunity cost is substantial: broadening Port this way delays polish on
the original narrow local tool. But the user objective has explicitly changed,
and the competitive comparison makes the old scope insufficient. Avoiding the
work would leave Port structurally behind on the exact axes that matter for a
hosted product [SRC-01][SRC-03][SRC-04].

### Dependencies

Port now depends on a sequence of architectural shifts:

- a new substrate-aware model beyond the current provider-only framing
  [SRC-01][SRC-02][SRC-04];
- a hosted control plane with API/SDK and machine lifecycle ownership
  [SRC-01][SRC-03];
- richer runtime manifests and inventory/status semantics [SRC-01][SRC-03];
- artifact distribution contracts for local and remote use [SRC-01][SRC-04];
- dedicated research and prototype slices for protected virtualization and Apple
  Virtualization Framework [SRC-05][SRC-06][SRC-08].

### Alternatives Considered

Alternatives considered:

- Keep PVM scoped out again:
  rejected because cloud cost control is now a primary product objective
  [SRC-01][SRC-04].
- Treat arm64 as proof that Firecracker PVM is already solved:
  rejected because current public evidence does not support that conclusion
  [SRC-05][SRC-06][SRC-08].
- Focus only on control-plane ergonomics and ignore substrate work:
  rejected because hosted value depends on having cost-effective execution lanes
  [SRC-01][SRC-02][SRC-04].
- Focus only on PVM and defer lifecycle/API expansion:
  rejected because Port still needs the control-plane scaffolding that would
  make any new substrate usable as a product [SRC-01][SRC-03].

## Recommendation

- [x] Proceed → create a new expansion epic and begin implementation from hosted-control and lifecycle foundations [SRC-01][SRC-02][SRC-03][SRC-04][SRC-05][SRC-06][SRC-08]
- [ ] Park → revisit later [SRC-02]
- [ ] Decline → document learnings [SRC-05][SRC-06]

Proceed with a multi-lane strategy:

1. Keep Firecracker with normal KVM as the primary proven lane.
2. Reopen PVM as an explicit product lane, starting with a dedicated Firecracker
   PVM track and treating arm64 protected virtualization as adjacent but not
   assumed solved.
3. Introduce substrate-aware planning for Cloud Hypervisor and Apple
   Virtualization Framework.
4. Start implementation from the hosted control-plane foundation, machine
   lifecycle surfaces, and artifact mobility, because every later substrate will
   need those product surfaces anyway [SRC-01][SRC-02][SRC-03][SRC-04][SRC-05][SRC-06][SRC-08].

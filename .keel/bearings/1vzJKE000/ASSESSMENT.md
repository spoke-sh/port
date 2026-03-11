---
id: 1vzJKE000
---

# Executable Pvm And Avf Lanes — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 5 | This work defines the next real execution lanes against the user's stated goal. |
| Confidence | 4 | The current repo state plus public substrate docs are strong enough to split execution cleanly. |
| Effort | 4 | Both programs are substantial, but each now has a bounded first slice. |
| Risk | 3 | The main risk is mixing Linux PVM and macOS AVF into one diluted queue. |

*Scores range from 1-5:*
- 1 = Very Low
- 2 = Low
- 3 = Medium
- 4 = High
- 5 = Very High

## Analysis

### Findings

- `x86_64` PVM host-kit delivery and AVF runtime delivery are both supportable,
  but they are materially different execution programs [SRC-01][SRC-02][SRC-05][SRC-06].
- arm64 cost control remains real through native-arm execution, while arm64
  Firecracker/PVM should stay research-only for now [SRC-03][SRC-04].

### Opportunity Cost

If Port keeps piling substrate work into one generic backlog, it will delay
both the primary Linux cost-control objective and the macOS first-class
objective. Splitting now costs some planning time but prevents the next voyages
from being incoherent [SRC-01][SRC-02][SRC-05][SRC-06].

### Dependencies

- The x86_64 PVM program depends on Port's existing hosted placement, node
  agent, and artifact seams [SRC-05].
- The AVF program depends on the existing shared machine and guest model, not
  on Linux PVM host-kit delivery [SRC-06].

### Alternatives Considered

Alternatives considered:

- Fold AVF under the Linux PVM program:
  rejected because the host platform, runtime owner, and operator proofs are
  materially different [SRC-02][SRC-05][SRC-06].
- Promote arm64 Firecracker/PVM into immediate execution:
  rejected because the current evidence still supports research-only status
  [SRC-03][SRC-04].
- Stop at documentation and admission gating:
  rejected because the board is empty but the user objective is not complete
  [SRC-05][SRC-06].

## Recommendation

- [x] Proceed → split into two execution epics [SRC-01][SRC-02][SRC-05][SRC-06]
- [ ] Park → revisit later [SRC-03]
- [ ] Decline → document learnings [SRC-04]

Proceed by creating:

- a Linux PVM execution epic for host-kit packaging, prepared-node launch, and
  hosted remote lifecycle [SRC-01][SRC-05]
- an AVF execution epic for macOS `doctor`, launch, guest transport, and
  console/log capture [SRC-02][SRC-06]

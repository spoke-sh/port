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

### Opportunity Cost

If Port keeps piling substrate work into one generic backlog, it will delay
both the primary Linux cost-control objective and the macOS first-class
objective. Splitting now costs some planning time but prevents the next voyages
from being incoherent.

### Dependencies

- The x86_64 PVM program depends on Port's existing hosted placement, node
  agent, and artifact seams.
- The AVF program depends on the existing shared machine and guest model, not
  on Linux PVM host-kit delivery.

### Alternatives Considered

Alternatives considered:

- Fold AVF under the Linux PVM program:
  rejected because the host platform, runtime owner, and operator proofs are
  materially different.
- Promote arm64 Firecracker/PVM into immediate execution:
  rejected because the current evidence still supports research-only status.
- Stop at documentation and admission gating:
  rejected because the board is empty but the user objective is not complete.

## Recommendation

- [x] Proceed → split into two execution epics
- [ ] Park → revisit later
- [ ] Decline → document learnings

Proceed by creating:

- a Linux PVM execution epic for host-kit packaging, prepared-node launch, and
  hosted remote lifecycle
- an AVF execution epic for macOS `doctor`, launch, guest transport, and
  console/log capture

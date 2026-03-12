---
id: VDcStSMlp
---

# K3s And Kubernetes Workloads — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 4 | A strong k3s lane would make Port much easier for humans to understand and evaluate. |
| Confidence | 3 | The outcome is clear, but the exact bootstrap and lifecycle design still depends on hybrid remote work. |
| Effort | 5 | This spans cluster bootstrap, node lifecycle, demo flows, and clear operator proof. |
| Risk | 4 | Kubernetes scope can expand quickly unless the first slice stays narrow. |

*Scores range from 1-5:*
- 1 = Very Low
- 2 = Low
- 3 = Medium
- 4 = High
- 5 = Very High

## Analysis

### Findings

- HA k3s is one of the best human-readable platform outcomes available [SRC-01].
- Port already has fleet and scheduler primitives that can be reused [SRC-03][SRC-04].
- The first slice should be a tightly scoped k3s workflow, not a generic
  Kubernetes product promise [SRC-01][SRC-02].

### Opportunity Cost

Choosing k3s too early could pull attention away from developer experience and
hybrid remote foundations. That risk is acceptable only if the first cluster
slice deliberately builds on those foundations instead of replacing them
[SRC-03][SRC-04].

### Dependencies

- Hosted-fleet and placement groundwork [SRC-03]
- Host-group and scheduler vocabulary [SRC-04]
- Human-facing cluster examples that define the expected outcome [SRC-01][SRC-02]

### Alternatives Considered

- Keep Kubernetes out of scope entirely. Rejected because the user explicitly
  wants first-class k3s support and external precedent shows it is a compelling
  outcome for VM platforms [SRC-01].
- Treat Kubernetes as just another service template. Rejected because HA k3s
  needs cluster lifecycle, node join, and operator proof work that is broader
  than one service definition [SRC-01][SRC-03].

## Recommendation

[ ] Proceed → convert to epic [SRC-01]
[x] Park → revisit later [SRC-01]
[ ] Decline → document learnings [SRC-01]

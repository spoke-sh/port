---
id: VDcStSMlp
---

# K3s And Kubernetes Workloads — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 4 | A strong K3s lane would make Port much easier for humans to understand and evaluate. |
| Confidence | 4 | The old sequencing blockers are now verified, and the repo has explicit hosted, installable, and proof-backed primitives to build on. |
| Effort | 4 | This still spans bootstrap, node lifecycle, and operator proof work, but the first slice can stay narrower than a full cluster platform. |
| Risk | 3 | Scope can still sprawl, but the current hosted, SSH, and storage boundaries make the first slice easier to contain. |

*Scores range from 1-5:*
- 1 = Very Low
- 2 = Low
- 3 = Medium
- 4 = High
- 5 = Very High

## Analysis

### Findings

- The earlier reasons to park K3s were sequencing objections, and the
  installable, hybrid-execution, and storage-foundation missions are now
  reflected in shipped install, operator, and hosted contracts [SRC-03][SRC-04][SRC-05].
- Port now has explicit hosted node, host-group, placement, and service
  contracts plus repo-local proofs that a first K3s lane can reuse [SRC-03][SRC-04][SRC-07].
- The first slice should be a hosted, stateless, tightly scoped K3s workflow,
  not an HA cluster or generic Kubernetes platform promise [SRC-01][SRC-02][SRC-05].

### Opportunity Cost

Continuing to park K3s would leave one of the clearest human-readable platform
outcomes unexplored even though the enabling substrate is now present. The real
trade is not "K3s or foundations" anymore; it is whether the first slice stays
narrow enough to defer HA control planes, persistent volumes, ingress, and SSH
parity while still proving Port can orchestrate a recognizable cluster outcome
[SRC-03][SRC-04][SRC-05].

### Dependencies

- Explicit installable, hybrid, and storage boundaries in the current product
  surface [SRC-03][SRC-04][SRC-05]
- Hosted node, host-group, placement, and service contracts with executable
  proof [SRC-04][SRC-07]
- Human-facing cluster examples that define the expected outcome [SRC-01][SRC-02]

### Alternatives Considered

- Keep K3s parked until HA storage and SSH guest or service parity exist.
  Rejected because the first slice can target a hosted, stateless cluster proof
  without those dependencies [SRC-04][SRC-05][SRC-07].
- Treat Kubernetes as just another service template. Rejected because even a
  narrow K3s slice needs cluster bootstrap, node join, kubeconfig or API
  exposure, and operator proof work that is broader than one service definition
  [SRC-01][SRC-04].
- Jump directly to HA or multi-provider K3s. Rejected because that would pull
  storage, ingress, and broader lifecycle work into the first slice before the
  narrower hosted contract is proven [SRC-02][SRC-05][SRC-07].

## Recommendation

[x] Proceed → convert to epic [SRC-01][SRC-03][SRC-04][SRC-05][SRC-07]
[ ] Park → revisit later [SRC-01]
[ ] Decline → document learnings [SRC-01]

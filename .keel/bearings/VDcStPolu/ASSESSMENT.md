---
id: VDcStPolu
---

# Hybrid Local Remote And SSH Execution — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 5 | This is the core product bridge between local adoption and real cloud or fleet usage. |
| Confidence | 4 | The repo already ships provider-aware docs, hosted transport, and an explicit SSH modeling seam. |
| Effort | 4 | The work spans bootstrap, auth, runtime ownership, and operator workflow design. |
| Risk | 3 | The main risk is splitting SSH and hosted control into competing models. |

*Scores range from 1-5:*
- 1 = Very Low
- 2 = Low
- 3 = Medium
- 4 = High
- 5 = Very High

## Analysis

### Findings

- The hybrid foundation already exists in the current control-plane and cloud
  docs [SRC-01][SRC-02].
- SSH is explicitly present as the next seam to productize rather than a new
  idea to invent from scratch [SRC-03][SRC-04].
- This work preserves the user's preferred local-plus-remote toolchain shape
  without requiring a second command family [SRC-01][SRC-02].

### Opportunity Cost

Pursuing hybrid execution first delays some higher-level workload work such as
k3s or GPU orchestration, but those features are less credible until the base
remote and SSH ownership model is first-class [SRC-02][SRC-04].

### Dependencies

- Provider-aware remote modeling and current hosted workflows [SRC-01]
- Hosted control-plane and node-agent ownership contract [SRC-02]
- Existing SSH seam in earlier planning artifacts [SRC-03]

### Alternatives Considered

- Focus only on hosted control-plane refinements and leave SSH for later.
  Rejected because the user explicitly wants first-class remote usage over SSH
  and the repo already models SSH as the intended seam [SRC-03][SRC-04].
- Add a separate remote-only CLI surface. Rejected because the current product
  direction has repeatedly preserved one canonical CLI and guest vocabulary
  across ownership modes [SRC-01][SRC-02].

## Recommendation

[x] Proceed → convert to epic [SRC-01]
[ ] Park → revisit later [SRC-01]
[ ] Decline → document learnings [SRC-01]

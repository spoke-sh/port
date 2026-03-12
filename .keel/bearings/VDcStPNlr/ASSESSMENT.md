---
id: VDcStPNlr
---

# GPU Execution Support — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 4 | GPU support opens materially different workloads and raises the ceiling of the platform. |
| Confidence | 2 | The direction is appealing, but the exact first substrate, hardware boundary, and operator proof are still open. |
| Effort | 5 | This spans host capability modeling, placement, runtime support, and product proof. |
| Risk | 5 | Hardware-specific work has high variance and can distract from nearer-term productization. |

*Scores range from 1-5:*
- 1 = Very Low
- 2 = Low
- 3 = Medium
- 4 = High
- 5 = Very High

## Analysis

### Findings

- GPU support is not present in the current product contract and needs explicit
  scoping before implementation [SRC-01].
- External precedent suggests a narrow passthrough-first lane instead of a
  generic accelerator promise [SRC-02][SRC-03].
- The highest-value GPU story likely depends on hosted placement and
  higher-level workload orchestration rather than standing alone [SRC-03][SRC-04].

### Opportunity Cost

Choosing GPU work too early would delay developer experience and hybrid-remote
foundations that enable broader adoption. That trade only makes sense after the
core product surface is easier to install and operate [SRC-01][SRC-04].

### Dependencies

- Explicit host capability and placement modeling [SRC-04]
- A narrow accelerator reference lane from external precedent [SRC-02][SRC-03]
- Better product distribution and hybrid remote operation so the resulting lane
  is usable outside this repo [SRC-01]

### Alternatives Considered

- Claim generic GPU support immediately. Rejected because there is no current
  Port contract for accelerators, and the external examples all rely on sharp
  platform and substrate boundaries [SRC-01][SRC-02].
- Ignore accelerators until much later. Rejected because the user explicitly
  wants GPU support on the horizon and it affects how we should think about
  storage, placement, and k3s follow-on work [SRC-03][SRC-04].

## Recommendation

[ ] Proceed → convert to epic [SRC-01]
[x] Park → revisit later [SRC-01]
[ ] Decline → document learnings [SRC-01]

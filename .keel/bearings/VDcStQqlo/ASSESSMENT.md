---
id: VDcStQqlo
---

# Cloud Block Storage Normalization — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 4 | Storage normalization is a foundational enabler for stateful cloud and k3s workloads. |
| Confidence | 3 | The need is clear, but the exact shared-model contract is still open. |
| Effort | 4 | This touches model, runtime, placement, and operator workflow design. |
| Risk | 4 | Storage work can easily overexpand into a much broader platform problem. |

*Scores range from 1-5:*
- 1 = Very Low
- 2 = Low
- 3 = Medium
- 4 = High
- 5 = Very High

## Analysis

### Findings

- Current Port storage is artifact-first and rootfs-first rather than
  volume-first [SRC-01][SRC-02].
- External VM-platform precedent keeps storage modes explicit and operator
  visible [SRC-03][SRC-04].
- This work would create a cleaner foundation for later stateful workloads and
  k3s stories [SRC-01][SRC-03].

### Opportunity Cost

Pursuing storage normalization before the hybrid remote story settles could
force rework in hosted placement and remote-node ownership. Even so, the topic
needs to be recorded now because stateful workloads and cloud usage both depend
on it [SRC-02][SRC-03].

### Dependencies

- Current artifact and rootfs contract [SRC-01][SRC-02]
- Explicit backend lessons from storage-oriented VM platform docs [SRC-03][SRC-04]
- Hosted placement and future stateful workload planning [SRC-02]

### Alternatives Considered

- Continue treating guest images as the only storage surface. Rejected because
  that approach does not answer the user's request for normalized block storage
  or support stateful cloud workloads well [SRC-01][SRC-02].
- Jump directly to a full storage service or CSI-style design. Rejected because
  the repo first needs a smaller shared contract for machines and hosted lanes
  [SRC-03][SRC-04].

## Recommendation

[x] Proceed → convert to epic [SRC-01]
[ ] Park → revisit later [SRC-01]
[ ] Decline → document learnings [SRC-01]

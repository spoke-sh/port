---
id: 1vydg7000
---

# Cloud Linux and PVM Viability — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 4 | A credible cloud lane expands Port from a local demo into a portable operator workflow. |
| Confidence | 4 | Current provider documentation is strong enough to bound MVP scope even without live cloud runtime access. |
| Effort | 3 | Modeling remote Linux hosts and documenting support is moderate effort; live cross-cloud runtime parity would be much larger. |
| Risk | 2 | The main risk is overpromising provider support, which the research already narrows substantially. |

*Scores range from 1-5:*
- 1 = Very Low
- 2 = Low
- 3 = Medium
- 4 = High
- 5 = Very High

## Analysis

### Opportunity Cost

The main opportunity cost is delaying local-runtime depth. That cost is
acceptable only if the cloud lane stays narrow: remote Linux host modeling, a
partial implementation, and explicit documentation. Attempting full provider
runtime parity during MVP would crowd out the core local Linux acceptance gates.

### Dependencies

The cloud lane depends on three things:

- Port's host model must cleanly separate `local` versus `remote` Linux targets.
- CLI help and docs must teach operators that Firecracker still runs on Linux
  hosts even when their workstation is macOS or Windows.
- Runtime implementation must stop short of unsupported provider promises,
  especially on Azure and on confidential/protected VM offerings.

### Alternatives Considered

Alternatives considered:

- Keep PVM/confidential VM scope in MVP:
  rejected because current provider support does not justify it.
- Treat every cloud as equally supported:
  rejected because Azure does not presently offer a supportable Firecracker path
  for MVP and provider support is materially different.
- Defer cloud work entirely:
  rejected because the MVP explicitly requires a documented design and partial
  implementation.

## Recommendation

- [x] Proceed → feed planning and implementation through the existing MVP epic
- [ ] Park → revisit later
- [ ] Decline → document learnings

Proceed with a partial cloud implementation that models remote Linux hosts,
targets AWS and GCP as the justified providers for future runtime proofs, and
documents Azure as unsupported for Firecracker MVP. Drop the PVM lane from MVP:
current provider documentation does not show a supportable overlap between
Firecracker's KVM requirements and protected/confidential VM offerings.

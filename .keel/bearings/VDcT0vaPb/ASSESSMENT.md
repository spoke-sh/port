---
id: VDcT0vaPb
---

# Installable Linux And Mac Developer Experience — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 5 | This is the highest-leverage way to make Port usable in many other projects without changing the core runtime model. |
| Confidence | 4 | The product gap is explicit in local docs, and the repo already has a clear Linux and Mac contract to package. |
| Effort | 3 | Packaging, release automation, and support-matrix work are substantial but bounded compared with new runtime substrates. |
| Risk | 3 | The main risks are overpromising platform support or shipping an incomplete Mac helper story. |

*Scores range from 1-5:*
- 1 = Very Low
- 2 = Low
- 3 = Medium
- 4 = High
- 5 = Very High

## Analysis

### Findings

- Release packaging is the shortest path from "credible repo" to "reusable
  product" [SRC-01].
- Linux and macOS already have enough published identity to support a real
  release matrix [SRC-02].
- The Mac AVF path needs productization work, not another conceptual runtime
  design pass [SRC-03].

### Opportunity Cost

Choosing this first means deferring deeper cloud or cluster features for a
short period, but that trade makes sense because broader platform and hosted
work will still be harder to adopt if Port continues to ship mainly as a repo
checkout [SRC-01].

### Dependencies

- Support-matrix decisions and release proof expectations from `RELEASE.md`
  [SRC-01]
- Current platform boundary from `README.md` [SRC-02]
- AVF helper and entitlement contract from `docs/avf.md` [SRC-03]

### Alternatives Considered

- Keep the source-first workflow as the default and postpone productization.
  Rejected because `RELEASE.md` already names the missing release work clearly,
  and the user objective is cross-project adoption now [SRC-01].
- Package Linux only. Rejected because README and `docs/avf.md` already make
  macOS a first-class lane, so leaving it out would preserve avoidable product
  asymmetry [SRC-02][SRC-03].

## Recommendation

[x] Proceed → convert to epic [SRC-01]
[ ] Park → revisit later [SRC-01]
[ ] Decline → document learnings [SRC-01]

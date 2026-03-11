---
id: 1vz3ck000
---

# PVM And Multi-Substrate Execution — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 5 | This bearing resets the product roadmap around real execution lanes instead of speculative future claims. |
| Confidence | 4 | Public substrate evidence and direct code inspection are strong enough to narrow the next planning move. |
| Effort | 4 | The follow-on work spans runtime, artifacts, hosted lifecycle ownership, and at least two substrate programs. |
| Risk | 4 | The main risk is overcommitting to unsupported architecture claims or mixing unrelated backends into one queue. |

*Scores range from 1-5:*
- 1 = Very Low
- 2 = Low
- 3 = Medium
- 4 = High
- 5 = Very High

## Analysis

### Findings

- `x86_64` Firecracker/PVM is a real execution lane, but it is a host-kit and
  runtime-ownership problem rather than a simple config flag [SRC-01][SRC-02][SRC-03].
- arm64 Firecracker/PVM should remain research-only, while native arm hardware
  remains a separate cost-control lane [SRC-02][SRC-06].
- AVF deserves a first-class macOS substrate track, and Port still needs a
  substrate-driver boundary in the runtime to support that split [SRC-04][SRC-05][SRC-07].

### Opportunity Cost

If Port keeps broad future-lane planning bundled together, it delays the
runtime and control-plane changes needed to make any of those lanes real. The
cost of splitting now is modest compared with the risk of another round of
documentation-only planning [SRC-01][SRC-04][SRC-07].

### Dependencies

- Port needs a substrate-driver boundary in the runtime before multiple
  execution programs can land cleanly [SRC-05][SRC-07].
- Port needs a hosted lifecycle contract above that boundary so local and remote
  ownership stop sharing one Firecracker-local critical path [SRC-05][SRC-07].
- Port needs an `x86_64` PVM host-kit plan covering kernel, Firecracker, and
  artifact variants [SRC-01][SRC-02][SRC-03].
- Port needs an AVF-specific implementation track for macOS operators
  [SRC-04][SRC-05].

### Alternatives Considered

- Keep treating provider-only planning as sufficient:
  rejected because provider labels do not solve runtime ownership, transport,
  or host-kit requirements [SRC-03][SRC-07].
- Promote arm64 Firecracker/PVM into immediate implementation:
  rejected because current reviewed evidence still supports research-only status
  [SRC-02][SRC-06].
- Drop AVF back to a documentation-only future lane:
  rejected because adjacent product and platform evidence support a first-class
  macOS substrate program [SRC-04][SRC-05].

## Recommendation

- [x] Proceed → plan execution backends around substrate drivers, hosted runtime ownership, `x86_64` PVM host kits, and an AVF macOS lane [SRC-01][SRC-04][SRC-05][SRC-07]
- [ ] Park → revisit later [SRC-06]
- [ ] Decline → document learnings [SRC-02]

Keep Firecracker/PVM for `x86_64` as a strategic execution lane. Keep Apple
Virtualization Framework as a first-class macOS substrate lane. Drop arm64
Firecracker/PVM from near-term implementation scope and keep it research-only
until there is stronger evidence than current public sources provide
[SRC-01][SRC-03][SRC-04][SRC-06].

Port should immediately plan the next implementation work around substrate
drivers and host/runtime ownership instead of more support-matrix prose
[SRC-05][SRC-07].

---
id: VDcStPolu
---

# Hybrid Local Remote And SSH Execution — Evidence

## Sources

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | manual | manual:doc-review | /home/alex/workspace/spoke-sh/port/docs/cloud.md | 2026-03-11 | 2026-03-11 | high | high | `docs/cloud.md` publishes the current provider-aware remote Linux and hosted execution boundary. |
| SRC-02 | manual | manual:doc-review | /home/alex/workspace/spoke-sh/port/docs/hosted.md | 2026-03-11 | 2026-03-11 | high | high | `docs/hosted.md` captures the canonical control-plane and node-agent ownership split and notes that remote launch is still follow-on work. |
| SRC-03 | manual | manual:doc-review | /home/alex/workspace/spoke-sh/port/.keel/epics/1vydg7000/voyages/1vyeq5000/SRS.md | 2026-03-11 | 2026-03-11 | high | high | Earlier planning artifacts explicitly treat the `ssh` host-connection model as the partial implementation seam for remote Linux hosts. |
| SRC-04 | manual | manual:doc-review | /home/alex/workspace/spoke-sh/port/.keel/epics/1vzXFf000/PRD.md | 2026-03-11 | 2026-03-11 | high | high | Direct SSH orchestration remained out of scope in the first hosted standard cloud launch slice. |

## Feasibility

Feasible. Port already has the right user-facing verbs and a live hosted
transport path. The main missing work is deciding how a first-class SSH lane
fits with that hosted ownership model instead of leaving remote usage as a
diagnosed-but-unimplemented boundary.

## Findings

### 1. Port already has most of the hybrid control shape

`docs/cloud.md` and `docs/hosted.md` show that Port already understands local
versus hosted runtime ownership, provider-aware remote identity, and the
control-plane plus node-agent split needed for hybrid workflows [SRC-01][SRC-02].

### 2. SSH is present as a seam, not yet as a product surface

Earlier planning artifacts explicitly used the `ssh` host-connection shape as
the correct seam for remote Linux hosts, but the first hosted cloud slice kept
direct SSH orchestration out of scope. That means SSH is the natural next
surface to productize if the goal is first-class hybrid operation [SRC-03][SRC-04].

### 3. Hybrid execution should unify ownership rather than fork the CLI

The current repo already succeeded by keeping one CLI model and one guest
protocol across local and hosted lanes. The SSH story should preserve that same
principle rather than introducing a second remote-only vocabulary [SRC-01][SRC-02].

## Open Technical Risks

- A direct SSH lane can drift away from the hosted node-agent ownership model
  if host bootstrap, guest attach, and auth are not deliberately aligned.
- Remote preflight can become confusing if `port doctor` does not clearly
  distinguish local host checks from remote-host readiness checks.
- SSH-first workflows may need an explicit console or serial fallback when the
  guest network is unhealthy.

## Key Findings

1. Port already has the right CLI and ownership vocabulary for hybrid work
   [SRC-01][SRC-02].
2. SSH is the obvious next seam to productize because it is already embedded in
   the remote-host planning contract [SRC-03][SRC-04].
3. The first hybrid slice should extend the current model instead of inventing
   a separate remote toolchain [SRC-01][SRC-02].

## Unknowns

- Should the first SSH workflow manage long-lived remote nodes, ad hoc hosts,
  or both?
- How much auth hardening should land before SSH-first remote usage is exposed
  as a top-level operator path?

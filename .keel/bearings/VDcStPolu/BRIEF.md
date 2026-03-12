# Hybrid Local Remote And SSH Execution — Brief

## Hypothesis

Port can become a credible hybrid execution tool if one canonical CLI and guest
model spans local machines, hosted control-plane execution, and SSH-first
remote Linux workflows without forcing operators into separate toolchains.

## Problem Space

Port already models remote Linux providers and ships a live hosted control
plane plus node-agent split, but direct remote SSH usage is still a boundary
instead of a first-class product surface. The user wants to deploy to the
cloud and operate across local plus remote environments with first-class SSH.

## Context

The current docs already prove several pieces of the hybrid story:

- `docs/cloud.md` publishes provider-aware remote Linux and hosted workflows,
- `docs/hosted.md` documents the node-agent and control-plane ownership split,
- earlier planning artifacts explicitly used the current `ssh` host-connection
  shape as the seam for remote work while leaving direct SSH orchestration out
  of scope.

That means the foundation exists, but the human-facing product contract is not
yet complete.

## Objectives

- Define the first-class hybrid execution contract across local, hosted, and
  SSH-first remote operation.
- Decide where direct SSH ownership ends and hosted node-agent ownership
  begins.
- Preserve one canonical `machine`, `guest`, and `artifacts` surface across
  those modes.
- Make bootstrap, auth, and diagnostics explicit enough to drive planning.

## Scope

- In scope: remote Linux over SSH, hosted and node-agent integration, operator
  workflows for local plus remote execution, and control-plane or node identity
  boundaries.
- Out of scope: multi-tenant SaaS rollout, cluster orchestration, or price-aware
  scheduling.

## Success Criteria

- [ ] The hybrid execution model names the canonical ownership patterns for
  local, hosted, and SSH-first remote operation.
- [ ] A first executable SSH-first workflow is concrete enough to plan as a
  voyage rather than a doc-only note.
- [ ] Auth, bootstrap, and `port doctor` behavior are explicit for remote
  Linux hosts.
- [ ] The research preserves one CLI and guest vocabulary instead of inventing
  a remote-only command family.

## Research Questions

- Should the first remote-host workflow be direct SSH, hosted-node bootstrap,
  or both?
- How should guest operations, logs, and file transfer behave when the runtime
  owner is reached over SSH?
- Which bootstrap state should live in config versus discovered remote
  inventory?

## Open Questions

- How much of the first-class SSH story belongs in `port` itself versus in a
  node-agent bootstrap helper?
- Should the first remote workflow start from ad hoc hosts or from registered
  managed nodes only?

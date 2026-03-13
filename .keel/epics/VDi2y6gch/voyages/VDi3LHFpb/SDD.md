# Hosted HTTP App Curl Proof - Software Design Description

> Make the repo-level proof surface host one minimal HTTP app through Port,
> curl it from the host, and record a human-reviewable artifact.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage turns the app-hosting proof epic into one bounded delivery slice.
It does not try to ship a general hosted application platform. Instead, it
reuses the existing hosted control plane, node agent, service runtime, and
guest-forward path to launch one minimal HTTP service, curl it from the host,
and publish a single reviewable proof surface.

## Context & Boundaries

### In Scope

- repo-local hosted control-plane and node-agent proof lane
- one minimal hosted HTTP application launched through `port service apply`
- host-side exposure through `port guest forward`
- one host-side `curl` assertion and one human-reviewable artifact
- repo-level proof-surface wiring and docs for this canonical path

### Out of Scope

- renaming the live surface to `screen`
- migrating the recorder path to `atxt`
- ingress, public networking, autoscaling, or multi-service orchestration
- external hosted infrastructure beyond the repo-local demo lane

```
┌──────────────────────────────────────────────────────────────────────┐
│                    Hosted HTTP App Curl Proof                       │
│                                                                      │
│  hosted service apply ───┐                                           │
│                          ├──> hosted guest forward ───> host curl     │
│  repo-level proof entry ─┘                    │                      │
│                                               └──> recording-backed   │
│                                                   proof artifact      │
└──────────────────────────────────────────────────────────────────────┘
            ↑                           ↑
   hosted control plane + node     docs / mission surface
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `port control-plane serve` and `port node-agent serve` | internal CLI/runtime | provide the repo-local hosted lane that owns service and guest-forward operations | current workspace |
| hosted `port service apply|list|status|stop` path | internal CLI/runtime | launch and observe the minimal hosted HTTP application | current workspace |
| hosted `port guest forward` path | internal CLI/runtime | expose the hosted app to the host for curl verification | current workspace |
| mission report / repo-level proof surface | repo workflow | present the canonical proof command and artifact from one place | current workspace |
| recording-backed proof renderer pattern | repo workflow | publish a human-reviewable artifact while direct `vhs` capture remains environment-sensitive | current workspace |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| First app shape | one minimal HTTP service returning one deterministic payload | smallest honest proof that Port can host and expose an app |
| Hosted lane | repo-local hosted control plane + node agent | already-shipped substrate with canonical service and forwarding surfaces |
| Exposure path | canonical `port guest forward` followed by host-side `curl` | proves real app reachability without introducing a second demo-only exposure model |
| Proof surface | current repo-level mission surface now, future `screen` later | ships the critical path now without blocking on upstream Keel command work |
| Human artifact | recording-backed renderer output | matches the repository's reliable current proof pattern |

## Architecture

The voyage introduces three coordinated layers:

1. hosted HTTP app proof workflow
2. repo-level proof-surface integration
3. docs and boundary publication

## Components

### Hosted HTTP App Proof Workflow

- Purpose: launch one minimal hosted HTTP service and prove it is reachable.
- Interface: hosted `port service apply`, `service status`, `guest forward`,
  and host-side `curl`.
- Behavior: stand up the repo-local hosted lane, apply one minimal HTTP
  process, expose it through Port, and assert the returned payload.

### Repo-Level Proof Surface

- Purpose: make the hosted app proof the primary operator-facing evidence for
  this epic.
- Interface: current `just mission` output, artifact gallery, and mission
  evidence paths.
- Behavior: render the proof command path and recorded artifact from one place
  until future `screen` cutover work lands.

### App Hosting Proof Docs

- Purpose: publish the canonical proof contract and stop it from implying a
  broader application platform than Port currently ships.
- Interface: README, focused docs, and story-linked proof evidence.
- Behavior: explain prerequisites, command flow, artifact review path, and
  future naming and recorder follow-ons.

## Interfaces

- `port control-plane serve --control-plane <name> --bind <addr>`
- `port node-agent serve --node <name> --bind <addr> --token <token>`
- `port service apply --machine <name> --name <name> --kind service -- <command...>`
- `port service status --machine <name> --name <name>`
- `port guest forward --machine <name> --listen <addr> --target <addr>`
- `curl http://<listen-addr>/...`
- `just mission [mission-id]` until future `screen` cutover work lands

## Data Flow

1. The proof workflow starts the repo-local hosted control plane and node
   agent.
2. The workflow applies one minimal HTTP service to a hosted machine through
   canonical `port service apply`.
3. Hosted service lifecycle starts the guest process under the selected node
   runtime root.
4. The workflow exposes the in-guest HTTP listener through canonical
   `port guest forward`.
5. The host runs `curl` against the Port-owned listener and validates the
   expected payload.
6. The proof renderer records a human-reviewable artifact, and the repo-level
   proof surface points maintainers to that artifact.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Hosted service fails to start | service status, logs, or command proof | fail the story with explicit hosted machine or node context | fix the service command or hosted runtime preconditions, then rerun |
| Hosted forward cannot expose the app | command proof or route-aware error | surface control-plane, node, and target detail | fix the target listener or hosted forward path, then retry |
| Host-side curl returns the wrong payload | command proof or recording review | fail the proof workflow instead of accepting partial app lifecycle success | correct app content or exposure wiring and rerun |
| Proof artifact drifts from the real workflow | proof review | regenerate from the canonical renderer path | keep artifact generation tied to story verification |
| Future `screen` or `atxt` expectations leak into the first slice | doc review or scope review | reject the drift and keep those items as follow-on work | create a dedicated future mission or routine-driven story instead |

## Story Decomposition

1. Workflow story: implement the hosted HTTP app launch, exposure, and curl
   proof path.
2. Surface story: wire the repo-level proof output to this canonical path and
   artifact.
3. Docs story: publish proof contract, prerequisites, and boundaries.

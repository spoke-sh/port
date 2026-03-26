# Hosted External Project Deployment Proof - Software Design Description

> Prove one external static-site project can be staged into hosted Port compute with guest copy, started through service apply, exposed through guest forward, and reviewed through the repo-level mission surface.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage turns the current hosted app proof into one bounded external-
project deployment slice. It does not try to ship app bundles, images, or a
general container-like platform. Instead, it reuses the already-shipped hosted
control plane, node agent, guest-copy transport, service runtime, and
guest-forward path to stage one external project snapshot into hosted compute,
run it, curl it from the host, and publish one reviewable proof surface.

## Context & Boundaries

### In Scope

- repo-local hosted control-plane and node-agent proof lane
- one external static-site project snapshot sourced outside the inline demo
- one canonical staging path through hosted `guest copy` and any minimal setup
- one hosted service runtime path through `port service apply`
- one host-side exposure through `port guest forward`
- one host-side `curl` assertion and one human-reviewable artifact
- repo-level proof-surface wiring and docs for this deployment slice

### Out of Scope

- app bundle artifact or runtime contracts
- projects that require additional guest language runtimes or package managers
- ingress, public networking, autoscaling, or multi-service orchestration
- renaming the live surface to `screen`
- migrating the recorder path to `atxt`

```
┌──────────────────────────────────────────────────────────────────────┐
│               Hosted External Project Deployment Proof              │
│                                                                      │
│  external project snapshot ──> hosted guest copy/setup ──┐           │
│                                                           ├──> hosted │
│  repo-level proof entry ──────────────────────────────────┘    service│
│                                                               runtime │
│                                                               │       │
│                              hosted guest forward ────────────┤       │
│                                                               v       │
│                                                       host curl +     │
│                                                  recording-backed     │
│                                                     proof artifact    │
└──────────────────────────────────────────────────────────────────────┘
            ↑                                            ↑
   repo-local source snapshot                    docs / mission surface
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `port control-plane serve` and `port node-agent serve` | internal CLI/runtime | provide the repo-local hosted lane that owns guest-copy, service, and guest-forward operations | current workspace |
| hosted `port guest copy` path | internal CLI/runtime | stage project bytes into hosted compute without assuming node-visible host paths | current workspace |
| hosted `port guest exec` path | internal CLI/runtime | perform any minimal unpack/setup step after staged bytes arrive | current workspace |
| hosted `port service apply|status|stop` path | internal CLI/runtime | launch and observe the staged external project as a managed process | current workspace |
| hosted `port guest forward` path | internal CLI/runtime | expose the staged project to the host for curl verification | current workspace |
| mission report / repo-level proof surface | repo workflow | present the canonical proof command and artifact from one place | current workspace |
| recording-backed proof renderer pattern | repo workflow | publish a human-reviewable artifact while direct `vhs` capture remains environment-sensitive | current workspace |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| First project shape | one BusyBox-compatible external static-site project snapshot | smallest honest proof that Port can deploy outside project content without adding new guest runtimes |
| Staging path | deterministic host-side snapshot copied through hosted `guest copy`, with optional hosted `guest exec` setup | exercises the shipped external-byte transport instead of creating files inline inside the service command |
| Runtime path | canonical `port service apply` owns the long-running HTTP process; setup stays separate from service lifecycle | keeps deployment proof attached to the shipped managed-process surface |
| Proof surface | current repo-level mission surface now, future `screen` later | ships the critical path now without blocking on upstream Keel command work |
| Human artifact | recording-backed renderer output | matches the repository's reliable current proof pattern |

## Architecture

The voyage introduces three coordinated layers:

1. external-project staging and runtime workflow
2. repo-level proof-surface integration
3. docs and boundary publication

## Components

### External Project Staging Workflow

- Purpose: move real project bytes into hosted compute and make them runnable.
- Interface: hosted `port guest copy`, optional hosted `port guest exec`.
- Behavior: package or point at a deterministic external snapshot on the host,
  copy it into the guest workspace, and prepare the staged directory for the
  service runtime.

### Hosted Service and Exposure Proof

- Purpose: run the staged project as a managed hosted service and prove it is
  reachable.
- Interface: hosted `port service apply`, `service status`, `guest forward`,
  and host-side `curl`.
- Behavior: start the repo-local hosted lane, run the project-serving command
  against the staged directory, expose it through Port, and assert the
  returned payload.

### Repo-Level Proof Surface

- Purpose: make the external-project deployment proof the primary
  operator-facing evidence for this epic.
- Interface: current `just mission` output, artifact gallery, and mission
  evidence paths.
- Behavior: render the proof command path and recorded artifact from one place
  until future `screen` cutover work lands.

### External Deployment Docs

- Purpose: publish the canonical deployment contract and stop it from implying
  a broader app-bundle platform than Port currently ships.
- Interface: README, focused docs, and story-linked proof evidence.
- Behavior: explain prerequisites, command flow, artifact review path, and the
  explicit boundary between this slice and future app-bundle missions.

## Interfaces

- `port control-plane serve --control-plane <name> --bind <addr>`
- `port node-agent serve --node <name> --bind <addr> --token <token>`
- `port guest copy --machine <name> --direction host-to-guest --source <path> --destination <path>`
- `port guest exec --machine <name> -- <command...>`
- `port service apply --machine <name> --name <name> --kind service -- <command...>`
- `port service status --machine <name> --name <name>`
- `port guest forward --machine <name> --listen <addr> --target <addr>`
- `curl http://<listen-addr>/...`
- `just mission [mission-id]` until future `screen` cutover work lands

## Data Flow

1. The proof workflow starts the repo-local hosted control plane and node
   agent.
2. The workflow prepares a deterministic external-project snapshot on the host.
3. Hosted `port guest copy` stages that snapshot into the hosted guest
   workspace.
4. If needed, hosted `port guest exec` performs one minimal unpack or setup
   step against the staged files.
5. Hosted `port service apply` starts the HTTP process against the staged
   project directory under the selected node runtime root.
6. Hosted `port guest forward` exposes the in-guest listener to the host.
7. The host runs `curl` against the Port-owned listener and validates the
   expected payload.
8. The proof renderer records a human-reviewable artifact, and the repo-level
   proof surface points maintainers to that artifact.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Hosted guest copy fails or stages incomplete bytes | command proof, copy error, or post-copy inspection | fail the story with explicit hosted route and machine context | fix the source snapshot or copy/setup logic, then rerun |
| Staged project needs unexpected runtime support | service status, logs, or setup failure | reject the sample project as out of scope for this voyage | switch back to a BusyBox-compatible external snapshot and rerun |
| Hosted service fails to start against the staged directory | service status, logs, or command proof | fail the story with explicit machine, host group, and node context | fix the service command or staged layout, then rerun |
| Hosted forward cannot expose the app | command proof or route-aware error | surface control-plane, node, and target detail | fix the target listener or hosted forward path, then retry |
| Host-side curl returns the wrong payload | command proof or recording review | fail the proof workflow instead of accepting partial deployment success | correct staged content or exposure wiring and rerun |
| Docs or proof surface imply app-bundle capabilities | doc review or scope review | reject the drift and keep bundle/runtime work as follow-on | create a dedicated future mission for app-bundle work |

## Story Decomposition

1. Workflow story: implement external-project staging, hosted runtime, and curl
   proof path.
2. Surface story: wire the repo-level proof output to this canonical workflow
   and artifact.
3. Docs story: publish deployment contract, prerequisites, and boundaries.

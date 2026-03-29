# Boot Live Local Cluster And Fix Packaged Guest Validation - Software Design Description

> Make the shipped local single-node cluster lane boot live on Linux, hand off
> a usable kubeconfig, and make guest artifact validation work from the
> installed Port contract.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage does not introduce a new operator surface. It hardens the one Port
just shipped by fixing the runtime and packaging seams underneath it. The work
centers on two defects that block downstream use today:

1. the shipped local guest lane panics during boot before `/init` runs cleanly
2. the packaged guest artifact validate path still resolves scripts from
   build-time-only `/build/...` paths

The design keeps the cluster-first CLI intact while repairing the guest image,
runtime launch path, and artifact validation contract so the checked-in example
becomes a live handoff instead of only a proof-backed surface.

## Context & Boundaries

### In Scope

- local single-node guest boot health for the shipped cluster lane
- live readiness and kubeconfig handoff validation
- install-safe packaged guest artifact validation
- downstream handoff verification against `spoke infra`

### Out of Scope

- AWS or hosted cluster work
- multi-node local or cross-node networking
- recorder or proof-recorder migration
- pushing bootstrap back into downstream `guest exec` workflows

```
┌─────────────────────────────────────────────────────────────────┐
│          Healthy Local Cluster Runtime And Artifact Lane        │
│                                                                 │
│  shipped guest image + boot wiring ───────┐                     │
│                                            ├──> local cluster    │
│  cluster runtime health + kubeconfig ─────┤      boot + handoff  │
│                                            │                     │
│  packaged artifact validate contract ──────┘                     │
└─────────────────────────────────────────────────────────────────┘
             ↑                           ↑
       Port runtime/model          downstream infra handoff
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `examples/port.toml` local cluster contract | internal config | canonical shipped cluster example to repair | current workspace |
| local Firecracker runtime and guest image artifacts | internal runtime | execute the actual single-node local cluster path | current workspace |
| artifact validation script resolution in `port-runtime` | internal runtime | keep packaged guest validation install-safe | current workspace |
| downstream `spoke infra` cluster handoff expectation | external consumer | confirm Port hands off a usable healthy cluster plus kubeconfig | current local checkout contract |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Surface ownership | keep `port cluster up/status/kubeconfig` as the only blessed handoff | the problem is runtime correctness, not missing CLI vocabulary |
| Runtime focus | fix guest image or boot wiring instead of adding more proof-only scaffolding | downstream failure happens before handoff; docs do not unblock it |
| Artifact fix | make validate-path resolution install-safe rather than assuming a source checkout | packaged Port must honor the same artifact contract it publishes |
| Scope guard | keep the mission local single-node only | AWS and multi-node are still premature until the first lane is healthy |

## Architecture

The voyage touches three cooperating layers:

1. local guest artifact and boot path
2. cluster runtime readiness and kubeconfig handoff
3. packaged artifact validate resolution

## Components

### Local Guest Artifact And Boot Path

- Purpose: make the shipped guest image boot cleanly through the current local
  Firecracker lane.
- Interface: existing local machine and cluster launch flow anchored by
  `examples/port.toml`.
- Behavior: ensure the guest image, kernel args, staged bootstrap inputs, and
  runtime launch configuration are consistent enough for `/init` to run and the
  guest agent or cluster bootstrap path to come up.

### Cluster Runtime Readiness And Kubeconfig Handoff

- Purpose: prove that the fixed local cluster lane reaches Port-owned healthy
  status and hands kubeconfig off directly.
- Interface: `port cluster up`, `port cluster status`, and
  `port cluster kubeconfig`.
- Behavior: launch the cluster, evaluate readiness, return machine and
  kubeconfig state, and keep downstream consumers out of manual API forwarding
  or rewrite workflows.

### Packaged Artifact Validate Resolution

- Purpose: make guest artifact validation work from installed or packaged Port
  binaries, not only source checkouts.
- Interface: existing `artifacts validate` contract for `demo-guest`.
- Behavior: resolve the validate script and its dependencies from a shipped or
  runtime-safe location before invoking the validation pipeline.

## Interfaces

- `port --config examples/port.toml cluster up --cluster demo --runtime-root <tmp> --format json`
- `port --config examples/port.toml cluster status --cluster demo --runtime-root <tmp> --format json`
- `port --config examples/port.toml cluster kubeconfig --cluster demo --runtime-root <tmp> --format json`
- `port --config examples/port.toml artifacts validate --artifact demo-guest --architecture x86-64`
- downstream `kubectl` or `spoke infra` consumption of the returned kubeconfig

## Data Flow

1. Operator runs the shipped local cluster workflow against `examples/port.toml`.
2. Port launches the local guest with the configured kernel, guest image, and
   cluster bootstrap contract.
3. The repaired guest lane reaches a clean `/init` path and completes cluster
   bootstrap.
4. Port evaluates readiness and returns status plus kubeconfig.
5. Downstream tooling consumes the returned kubeconfig directly.
6. In parallel, the installed artifact validate path resolves its validation
   contract without depending on `/build/...` source paths.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Guest image still panics during boot or `/init` fails | `cluster up` failure plus guest console logs | fail with explicit runtime paths and preserve console evidence | repair guest artifact, boot wiring, or runtime staging and rerun |
| Cluster boots but never becomes ready | `cluster status` readiness remains non-ready | report unhealthy cluster state without hiding it behind downstream work | inspect bootstrap logs and readiness checks, then retry |
| Returned kubeconfig still requires rewrite or fails with `kubectl` | host-side handoff proof | keep the story open and treat handoff as incomplete | fix server endpoint or forward ownership inside Port |
| Packaged artifact validation still resolves source-only paths | `artifacts validate` failure mentioning `/build/...` or missing shipped scripts | fail fast and keep install-safety gap explicit | move scripts into shipped paths or resolve them relative to the install contract |
| Scope drifts into AWS or multi-node work | planning or implementation review | reject the change from this voyage | move follow-on work into a later mission |

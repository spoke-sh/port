# Replace Demo API With GitOps-Capable Local K3s Runtime - Software Design Description

> Upgrade Port's local single-node cluster lane from a demo API to a real
> GitOps-capable K3s control plane that supports normal kubeconfig handoff,
> Kubernetes discovery, Flux install, and Helm operator install.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage replaces the last demo-only seam in Port's local cluster lane. The
existing `cluster up/status/kubeconfig` surface is already the right operator
contract, but the runtime underneath it still behaves like a constrained demo:
API discovery is incomplete and GitOps bootstrap clients do not yet succeed
reliably against the handed-off kubeconfig.

The design keeps the public cluster-first CLI intact and upgrades the runtime
behind it in three coordinated slices:

1. replace the local demo control-plane path with a real single-node K3s boot
2. harden kubeconfig handoff and API reachability for normal clients
3. prove Flux and Helm operator installation against the handed-off kubeconfig

## Context & Boundaries

### In Scope

- real local single-node K3s runtime behavior for the shipped `demo` cluster
- kubeconfig handoff and API reachability owned by Port
- Kubernetes API discovery needed by Flux, Helm, and operator bootstrap
- direct host-side GitOps bootstrap proof

### Out of Scope

- AWS, hosted-cluster, or multi-node orchestration
- ingress or load balancer work beyond what a functioning local control plane
  strictly requires
- proof-recorder changes, downstream `infra` verification, or downstream
  workarounds that bypass Port's cluster contract

```
┌────────────────────────────────────────────────────────────────┐
│              GitOps-Ready Local Cluster Runtime                │
│                                                                │
│  local K3s boot ───────────────┐                               │
│  kubeconfig + API reachability ├──> real control plane handoff │
│  Flux/Helm bootstrap proof ────┤                               │
│  unchanged infra proof ────────┘                               │
└────────────────────────────────────────────────────────────────┘
              ↑                                   ↑
        Port runtime/model                  downstream infra
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `examples/port.toml` `demo` cluster contract | internal config | canonical shipped local cluster workflow to harden | current workspace |
| local guest artifacts and offline K3s bootstrap kit | internal runtime | bring up the actual single-node K3s control plane | current workspace |
| `kubectl`, `flux`, and `helm` host-side clients | external tools | validate the handed-off kubeconfig against real Kubernetes and GitOps clients | current workspace toolchain |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Surface ownership | Keep `port cluster up/status/kubeconfig` as the only blessed control-plane handoff surface. | The gap is runtime depth, not missing CLI vocabulary. |
| Runtime target | Replace demo behavior with a real single-node K3s control plane rather than adding more special-case API emulation. | Flux, Helm, and downstream infra need a real Kubernetes API. |
| Proof boundary | Require Flux and Helm verification in addition to Port-local checks. | GitOps readiness is only credible if real host-side clients succeed against the handed-off kubeconfig. |
| Scope guard | Stay local-only and single-node only. | Networking, AWS, and multi-node concerns are follow-on missions and would dilute this runtime-correctness slice. |

## Architecture

The voyage touches three cooperating layers:

1. local cluster bootstrap inputs and runtime behavior
2. Port-owned kubeconfig and API reachability handoff
3. host-side GitOps bootstrap client proof

## Components

### Real Local K3s Bootstrap

- Purpose: make the shipped `demo` cluster boot a real single-node K3s control
  plane instead of a stub API.
- Interface: existing `port cluster up` runtime path and checked-in bootstrap
  inputs under `examples/`.
- Behavior: stage and invoke the offline K3s bootstrap kit, wait for the real
  control plane to become usable, and persist the runtime state needed for
  later status and kubeconfig calls.

### Kubeconfig And API Reachability Handoff

- Purpose: return a kubeconfig that normal host-side Kubernetes clients can use
  directly.
- Interface: `port cluster kubeconfig` and `port cluster status`.
- Behavior: own any forwarding or endpoint selection inside Port, return stable
  kubeconfig material, and report readiness based on real Kubernetes health
  rather than a demo signal.

### GitOps Bootstrap Proof Layer

- Purpose: verify the real control plane exposes the resource discovery and API
  behavior required by Flux and Helm.
- Interface: host-side `kubectl api-resources`, `flux install`, and
  `helm upgrade --install pulumi-kubernetes-operator ...`.
- Behavior: consume the Port-provided kubeconfig unchanged and fail loudly if
  any discovery or install prerequisite is missing.

## Interfaces

- `port --config examples/port.toml cluster up --cluster demo --runtime-root <tmp> --format json`
- `port --config examples/port.toml cluster status --cluster demo --runtime-root <tmp> --format json`
- `port --config examples/port.toml cluster kubeconfig --cluster demo --runtime-root <tmp> --format json`
- `kubectl --kubeconfig <path> api-resources -o name`
- `flux install --kubeconfig <path>`
- `helm upgrade --install pulumi-kubernetes-operator ... --kubeconfig <path>`

## Data Flow

1. Operator runs Port's checked-in `demo` local cluster workflow.
2. Port launches the local machine and stages the offline K3s bootstrap inputs.
3. The guest brings up a real single-node K3s control plane.
4. Port evaluates cluster readiness from the real control plane state and
   returns machine and kubeconfig data.
5. Host-side `kubectl`, `flux`, and `helm` consume the kubeconfig unchanged.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Local cluster still boots into a demo or incomplete API surface | `kubectl api-resources` misses required resource types or GitOps clients fail | keep the story open and treat the runtime as non-compliant | fix bootstrap inputs, runtime behavior, or readiness checks until real K3s behavior is present |
| Kubeconfig still requires downstream rewriting or manual forwarding steps | host-side client proof fails | fail the handoff explicitly rather than document a workaround | move endpoint selection and forwarding ownership back into Port |
| Flux or Helm install fails on missing API prerequisites | direct client command failure | keep GitOps readiness blocked and capture the missing resource or capability | repair control-plane runtime or install prerequisites in the guest lane |
| Work expands into AWS or multi-node concerns | implementation review | reject the change from this voyage | move the expansion into a later mission |

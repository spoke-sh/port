# Ship GitOps-Ready Local Cluster Runtime Contract - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver epic `VFHmKH5XR` so `port cluster up --cluster demo` produces a real single-node local K3s control plane rather than the current demo API surface. | board: VFHmKH5XR |
| MG-02 | `port cluster kubeconfig --cluster demo --format json` returns a kubeconfig that works for normal Kubernetes clients without downstream rewriting. | manual: use the handed-off kubeconfig with standard Kubernetes clients from the host |
| MG-03 | `kubectl api-resources -o name` against the handed-off kubeconfig includes at least `deployments.apps`, `namespaces`, `serviceaccounts`, `secrets`, `configmaps`, and `customresourcedefinitions.apiextensions.k8s.io`. | manual: inspect API discovery from the handed-off kubeconfig |
| MG-04 | `flux install` succeeds against the handed-off local kubeconfig. | manual: run `flux install` against the Port-provided kubeconfig |
| MG-05 | `helm upgrade --install pulumi-kubernetes-operator ...` succeeds against the same handed-off kubeconfig. | manual: run the operator install against the Port-provided kubeconfig |
| MG-06 | Downstream `spoke infra` proof passes unchanged through `infra bootstrap --env local` and `infra health --env local`. | manual: run the downstream repo proof unchanged |

## Constraints

- Scope stays local-only and single-node only.
- Do not add AWS, hosted-cluster, or multi-node orchestration in this mission.
- Do not add ingress or load-balancer work unless it is strictly required for a real local single-node K3s control plane to function.
- Keep Port as the owner of cluster bring-up, readiness, kubeconfig handoff, and API reachability; do not push GitOps prerequisites back into downstream guest choreography.

## Halting Rules

- DO NOT halt while the local cluster still behaves like a demo API or while Flux, Helm, or downstream `infra` bootstrap remain blocked on shortcomings in Port's local cluster contract.
- HALT when epic `VFHmKH5XR` is done and manual verification confirms normal Kubernetes API discovery, `flux install`, the Pulumi operator Helm install, and unchanged downstream `infra bootstrap` and `infra health`.
- YIELD if the remaining blocker requires a product decision on local guest networking, Kubernetes distribution choice, or non-local scope expansion rather than implementation work.

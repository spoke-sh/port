# Seal Healthy Local Cluster Runtime Contract - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver epic `VFH7YspJx` so the shipped local single-node cluster lane boots live, reports healthy status, returns a usable kubeconfig handoff, and fixes the packaged guest-artifact validation path. | board: VFH7YspJx |
| MG-02 | The checked-in example succeeds on Linux with `port --config examples/port.toml cluster up --cluster demo --runtime-root <tmp> --format json` instead of failing during Firecracker boot. | manual: run the shipped local cluster workflow on Linux and confirm `cluster up` succeeds |
| MG-03 | `port cluster status --format json` reports `readiness=ready`, `machine_state=running`, and `kubeconfig_available=true`, and `port cluster kubeconfig --format json` plus `kubectl get nodes -o wide` works without kubeconfig rewriting. | manual: inspect live local cluster status and kubeconfig handoff with downstream tooling |
| MG-04 | `port --config examples/port.toml artifacts validate --artifact demo-guest --architecture x86-64` succeeds from the installed or packaged CLI contract instead of resolving validation scripts under `/build/...`. | manual: run the shipped artifact validate path outside a source-build-only assumption |
| MG-05 | Scope stays single-node local only; no AWS, hosted cluster, or multi-node expansion lands in this mission. | manual: review planning and implementation artifacts for explicit single-node local boundaries |

## Constraints

- Fix runtime and artifact correctness, not proof UX, recorder migration, or broader docs polish.
- Do not shift cluster bootstrap or kubeconfig handoff back onto downstream `guest exec`, join-token choreography, or manual kubeconfig rewriting.
- Keep the mission bounded to the shipped single-node local cluster lane and packaged artifact path; defer AWS and multi-node expansion.

## Halting Rules

- DO NOT halt while the shipped local `cluster up/status/kubeconfig` workflow or packaged `artifacts validate` path still fail in the repo.
- HALT when epic `VFH7YspJx` is done and manual verification confirms a live healthy local cluster handoff plus install-safe guest artifact validation.
- YIELD if the remaining blocker requires a product decision on guest image provenance, Firecracker lane ownership, or downstream contract scope rather than implementation work.

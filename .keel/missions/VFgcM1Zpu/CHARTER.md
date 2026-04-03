# Seal Hosted AWS PVM Runtime Contract - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver a new epic that turns Port's prepared-node x86_64 Firecracker/PVM demo into a real provider-backed `cloud-aws` hosted lane. | board: VFgcPDfEj |
| MG-02 | On a prepared x86_64 AWS Linux node, canonical `port control-plane prepare-pvm-node`, `port machine launch --machine cloud-aws`, `port machine status --machine cloud-aws`, and `port machine stop --machine cloud-aws` succeed through the live hosted control-plane and node-agent path. | manual: run the live AWS hosted PVM workflow end to end |
| MG-03 | Port owns an explicit AWS host-kit preparation and readiness contract that proves custom kernel, `pti=off`, patched `firecracker-pvm`, and PVM artifact-kit availability without a manual config-overlay dance. | manual: prepare a node and inspect doctor / status / imported inventory output |
| MG-04 | Failure surfaces remain provider-aware and honest: missing host kit, wrong kernel, missing patched VMM, stale imported readiness, or missing PVM artifacts fail with actionable `cloud-aws` guidance and no fallback to the standard lane. | manual: inspect canonical failure paths plus automated tests |
| MG-05 | Scope stays x86_64 AWS hosted PVM only; arm64 remains research-only and GCP/Azure or broader scheduler rollout do not land in this mission. | manual: review planning and implementation artifacts for explicit AWS-only boundaries |

## Constraints

- Keep the canonical Port command model; do not add a second AWS-only command family.
- Port owns the runtime and host-preparation contract; EC2 provisioning, IAM, DNS, and downstream GitOps remain out of scope.
- Do not solve this by switching the target to bare metal or by silently reusing the standard Firecracker/KVM lane.
- Keep `cloud-aws` as the canonical operator proof surface, not `cloud-generic`.

## Halting Rules

- DO NOT halt while `cloud-aws` PVM still depends on generic-node substitution, manual imported-inventory edits, or hand-wired config overlays to demonstrate readiness.
- HALT when the new epic is done and a live AWS x86_64 prepared node proves canonical `prepare-pvm-node` plus `machine launch/status/stop` for `cloud-aws`, with provider-aware failure messaging.
- YIELD if the remaining blocker is a product decision about whether Port builds the AWS host kit itself or consumes an external host-kit artifact.

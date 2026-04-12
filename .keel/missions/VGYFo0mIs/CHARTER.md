# Raise Hosted AWS PVM Clusters To Real HA - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver epic `VGYFpfUph` so hosted AWS PVM clusters can place control-plane microVMs across distinct execution hosts instead of collapsing HA onto one host. | board: VGYFpfUph |
| MG-02 | Deliver epic `VGYFpfmpi` so the AWS PVM lane exposes a stable HA API endpoint, failover posture, and proof surface that remain usable after control-plane host loss. | board: VGYFpfmpi |

## Constraints

- Preserve the existing downstream `infra` contract on `port cluster up`,
  `status`, and `kubeconfig`; the provider-specific HA work belongs inside
  Port, not in downstream orchestration glue.
- Count HA only when control-plane microVMs are spread across distinct
  execution hosts behind a stable endpoint. Multiple control-plane guests on one
  host do not satisfy this mission.
- Keep AWS `x86_64` Firecracker/PVM as the first real-HA lane; do not broaden
  the promise to generic hosted, GCP, Azure, or arm64 PVM work.
- Preserve explicit failure surfaces: missing host capacity, unstable endpoint,
  or partial placement must fail honestly instead of silently degrading to the
  single-host story.
- Keep proof and inspection surfaces human-reviewable.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when epics `VGYFpfUph` and `VGYFpfmpi` are complete and the mission can
  be achieved.
- YIELD to human when the remaining blocker is an environment or provider
  decision about stable endpoint ownership, failure domain shape, or external
  load-balancer policy that cannot be resolved from repository context.

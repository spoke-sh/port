# Export Canonical Hosted Cluster Truth And Seal Managed Lifecycle Ownership - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver epic `VGcgU7q58` so Port exposes one canonical hosted-cluster status contract with machine, placement, managed-service, and legacy-runtime truth that downstream `infra` can consume directly. | board: VGcgU7q58 |
| MG-02 | Deliver epic `VGcgU9T57` so hosted K3s lifecycle stays under explicit Port-managed service ownership, including placement persistence, legacy-path rejection, and durability proof. | board: VGcgU9T57 |

## Constraints

- Preserve the downstream contract on `port cluster up`, `port cluster status`,
  and `port cluster kubeconfig`; the simplification belongs inside Port rather
  than in downstream orchestration glue.
- Keep hosted lifecycle ownership inside Port: placements, managed-service
  truth, restart posture, and runtime drift detection should not require
  `infra` reconstruction.
- Treat legacy detached K3s PID/log artifacts as invalid runtime drift rather
  than an alternate happy path for hosted clusters.
- Make the status contract machine-readable and explicit enough for the paired
  `infra` mission `VGcfT59ur` to consume without ad hoc probing.
- Keep proof human-reviewable, with a specific focus on the 60-90 minute worker
  stability failure mode seen in prod.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when epics `VGcgU7q58` and `VGcgU9T57` are complete and the mission can
  be achieved.
- YIELD to human when the remaining blocker is a cross-repo schema or product
  decision about hosted status semantics, proof expectations, or lifecycle
  boundaries that cannot be resolved from repository context.

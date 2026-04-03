# Raise AWS PVM Production Documentation Fidelity - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver epic `VFhICYohO` so Port's foundational and user-facing docs converge on one clear AWS production narrative with an explicit x86_64 hosted Firecracker/PVM lane. | board: VFhICYohO |
| MG-02 | Root contracts such as `README.md`, `ARCHITECTURE.md`, and `CONFIGURATION.md` make it obvious which AWS deployment path is strongest today, which lane is still a repo-local proof, and which boundaries remain explicit. | manual: review foundational docs for one coherent AWS/PVM production story |
| MG-03 | Focused guides and public docs explain AWS host-kit requirements, artifact-kit requirements, `prepare-pvm-node`, canonical machine launch/status/stop, and provider-aware failure surfaces without forcing operators to stitch together multiple contradictory pages. | manual: review `docs/hosted.md`, `docs/cloud.md`, `docs/pvm.md`, and the public AWS docs path |
| MG-04 | Scope stays documentation-only and truthful: no runtime claims beyond the current Port contract, no arm64 PVM promise, and no implied AWS infrastructure automation beyond Port's hosted lane. | manual: review planning and docs artifacts for explicit scope boundaries |

## Constraints

- Simplify the documentation map instead of adding another overlapping AWS narrative.
- Keep AWS x86_64 hosted PVM as the strongest production-oriented cloud story without hiding that the standard hosted lane still exists.
- Preserve current product truth: Port owns the runtime, host-kit, artifact-kit, and hosted control-plane contract, but not EC2 provisioning, IAM, VPC wiring, DNS, or downstream GitOps.
- Keep GCP, Azure, and arm64 PVM boundaries explicit rather than inheriting the AWS contract by implication.

## Halting Rules

- DO NOT halt while operators still need to read several conflicting pages to understand the AWS PVM production path.
- HALT when epic `VFhICYohO` is done and only manual mission verification remains.
- YIELD if the remaining blocker is a product decision about whether AWS `standard` or AWS `pvm` should be the primary public production narrative.

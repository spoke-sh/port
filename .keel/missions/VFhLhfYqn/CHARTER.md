# Export AWS PVM Host Kit As A First-Class Nix Module - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver epic `VFhLhfrqk` so Port exports a downstream-consumable AWS x86_64 PVM host-kit surface through its flake. | board: VFhLhfrqk |
| MG-02 | A downstream `nixosSystem { modules = [ port.nixosModules.aws-pvm-host ]; }` evaluates successfully and the resulting host config encodes the canonical AWS PVM host contract. | manual: evaluate the downstream nixosSystem import and inspect the derived host config |
| MG-03 | Port's exported Nix host-kit surface lines up with the canonical host-kit identity used by `prepare-pvm-node`, including package name, package version, host-kernel release, Firecracker build, boot args, and canonical `firecracker-pvm` path/env surface. | manual: inspect exported metadata/package surface and its relationship to the existing PVM host-kit contract |
| MG-04 | Port docs show the supported downstream AMI build handoff using the Port-owned module/package surface instead of a downstream repo-local host-kit module. | manual: review the updated AWS/Nix docs and downstream handoff example |
| MG-05 | Scope stays limited to Port-owned AWS x86_64 PVM host-kit definition surfaces. VM Import/Export automation, AMI publication policy, and downstream bootstrap orchestration remain outside Port. | manual: review planning and implementation artifacts for explicit boundaries |

## Constraints

- Keep the export shape AWS-specific and x86_64-specific for the PVM lane; do not generalize this into a multi-provider or arm64 surface.
- Keep the existing Port verb and readiness model. This mission exports the host-kit contract; it does not replace `prepare-pvm-node` or invent a second readiness workflow.
- Do not move AWS VM Import/Export automation, AMI publication, or downstream bootstrap policy into Port.
- Keep the module downstream-consumable without requiring a downstream repo to invent its own host-kit module path.

## Halting Rules

- DO NOT halt while the flake still lacks a first-class AWS PVM host-kit module/package surface or while downstream docs still require a repo-local custom host-kit module.
- HALT when the exported Nix surface, docs, and proof satisfy the mission goals and the board work is sealed.
- YIELD if the remaining blocker is an external product decision about the true patched-kernel or patched-VMM source Port should consume rather than export.

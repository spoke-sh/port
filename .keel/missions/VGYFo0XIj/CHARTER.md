# Implement Workspace Builder And Promotion Runtime Classes - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver epic `VGYFpewpf` so Port provides a canonical `workspace-scratch-builder` runtime class with isolated writable state, bounded trust posture, and reusable execution proof surfaces. | board: VGYFpewpf |
| MG-02 | Deliver epic `VGYFpf9pg` so Port provides a distinct clean-room promotion-runner runtime class and execution-proof contract for trusted closure publication across local and AWS lanes. | board: VGYFpf9pg |

## Constraints

- Preserve the ownership split already planned in Spoke and `infra`: Spoke owns
  creator-facing lineage and admission posture, `infra` owns builder/cache/
  publication substrate, and Port owns runtime execution and proof.
- Keep `workspace-scratch-builder` and
  `blessed-closure-promotion-runner` as distinct runtime classes; scratch
  state must never be elevated into trusted publication by mode bit or reuse.
- Keep the contract explicit through Port-authored runtime, machine, guest, or
  proof surfaces rather than shell-only conventions.
- Preserve local/AWS parity at the contract level even when the runtime
  substrate differs.
- Do not move creator-facing promotion policy, signing decisions, or cache
  ownership into Port.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when epics `VGYFpewpf` and `VGYFpf9pg` are complete and the mission can
  be achieved.
- YIELD to human when the remaining blocker is a platform decision about runtime
  isolation, trust-material mounting, or proof semantics that cannot be
  resolved from repository context.

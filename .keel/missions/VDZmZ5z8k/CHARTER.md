# Deliver Hosted Service Hardening - Charter

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Complete epic `1vzfT4000` and its remaining delivery work while keeping hosted-service hardening traceable through board-managed stories and voyages. | board: 1vzfT4000 |
| MG-02 | Maintain a doctor-clean planning board and capture major steering decisions in the mission log while the epic stays active. | manual: review mission log and board health at handoff |

## Constraints

- Keep `nix develop -c just keel doctor` clean after structural board changes.
- Use one atomic Conventional Commit per story or planning/governance slice.
- Do not add compatibility shims or legacy workflow aliases unless a story explicitly requires them.
- Preserve mission-to-epic traceability for any new child entities created under this workstream.

## Halting Rules

- DO NOT halt while epic `1vzfT4000` still has actionable or unblockable board work that can be progressed from local context.
- HALT when `board: 1vzfT4000` is satisfied and only manual verification or acceptance remains.
- YIELD to human when progress depends on unavailable credentials, external infrastructure, or acceptance steps that cannot be safely reproduced here.

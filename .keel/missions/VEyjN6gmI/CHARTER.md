# Ship Canonical External Project Deployment Workflow - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver epic VEyjUL2Zr so Port has one canonical repo-level proof that stages a real external static-site project snapshot into hosted compute with hosted `guest copy`, serves it through `port service apply`, curls it from the host, and records human-reviewable evidence. | board: VEyjUL2Zr |
| MG-02 | Keep the board doctor-clean and make the external-project deployment mission legible through mission, flow, and routine surfaces while keeping app-bundle work explicit follow-on scope. | manual: run `keel mission show VEyjN6gmI`, `keel mission next VEyjN6gmI`, `keel flow`, `keel routine show review-atxt-mission-proof-adoption`, and `keel doctor` |

## Constraints

- Keep the first shipped workflow narrow: one external static-site project
  snapshot, one staging path, one service process, one host-side curl, and one
  recording-backed proof artifact.
- Reuse shipped hosted primitives only: `port guest copy`, optional
  `port guest exec` for setup, `port service`, `port guest forward`, and the
  current repo-level mission proof surface.
- Keep the proof repo-local and reproducible; prefer a vendored or captured
  external-project snapshot over live network fetches during verification.
- Treat app bundle artifact contracts, app bundle runtimes, and
  language-specific runtime expansion as follow-on missions rather than hidden
  scope in this slice.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when VEyjUL2Zr is complete and only manual mission verification remains
- YIELD to human when progress depends on a product decision about what should
  qualify as the first app bundle contract or runtime semantics beyond current
  hosted primitives

# Ship Canonical App Hosting Screen Proof - Charter

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver epic VDi2y6gch so Port has one canonical repo-level app-hosting proof surface that launches a minimal HTTP app through the hosted path, curls it from the host, and records a human-reviewable artifact. | board: VDi2y6gch |
| MG-02 | Keep the board doctor-clean and make this app-hosting proof mission legible through mission, flow, and routine surfaces. | manual: run `keel mission show VDi2jvg4P`, `keel mission next VDi2jvg4P`, `keel flow`, `keel routine show VEz56fPp4`, and `keel doctor` |

## Constraints

- Keep the first shipped workflow narrow: one minimal hosted HTTP application, one host-side curl proof, and one human-reviewable recording path.
- Build on the already-shipped hosted `service apply` and hosted `guest forward` routes instead of inventing a second app-hosting surface or bypassing Port-managed compute.
- Use the current recorder path that works in this repository today, and treat `keel screen` plus `atxt` as follow-on upgrades rather than blockers for the first canonical proof.
- Preserve the hard-cutover policy: do not keep multiple long-term names or proof surfaces alive in parallel beyond an explicitly scoped migration slice.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when VDi2y6gch is complete and only manual verification remains
- YIELD to human when progress depends on a product decision about proof naming, recorder choice, or external publishing expectations that are not inferable from the repository

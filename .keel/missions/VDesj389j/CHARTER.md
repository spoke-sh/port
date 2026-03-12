# Ship Hybrid Local Remote And SSH Execution - Charter

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Convert bearing VDcStPolu into executable board work for hybrid local, remote, cloud, and SSH-first execution. | board: VDcStPolu |
| MG-02 | Keep the mission surfaces doctor-clean and make this hybrid-execution mission visible through the board command surfaces. | manual: run `just keel mission show VDesj389j`, `just flow`, and `just keel doctor` |

## Constraints

- Keep one canonical `port` CLI and guest vocabulary across local, hosted, and SSH-owned execution instead of introducing a separate remote-only command surface.
- Use bearing VDcStPolu plus the existing hosted, cloud, and operator artifacts as the primary sources, and do not broaden scope into storage normalization, k3s, or GPU work unless the hybrid contract requires explicit downstream follow-up.
- Preserve the hard-cutover policy: do not add compatibility aliases or dual operator paths just to bridge old and new remote-execution semantics.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when the hybrid execution work is decomposed into executable board items and only verification-oriented manual checks remain
- YIELD to human when prioritization depends on provider choice, SSH trust model, or remote bootstrap assumptions that are not inferable from the repository

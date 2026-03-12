# Reassess K3s And Kubernetes Workloads - Charter

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Reassess bearing VDcStSMlp against the now-verified installable developer experience, hybrid execution, and storage foundations, and convert that bearing into the next explicit board state for K3s work. | board: VDcStSMlp |
| MG-02 | Keep the mission surfaces doctor-clean and make this K3s reassessment visible through mission, bearing-list, and flow surfaces. | manual: run `just keel mission show VDfqti68W`, `just keel mission next VDfqti68W`, `just keel bearing list`, `just flow`, and `just keel doctor` |

## Constraints

- Keep the first K3s outcome narrow: one canonical operator workflow and one bounded cluster or service slice, not a generic Kubernetes platform promise.
- Use bearing VDcStSMlp plus the now-verified installable, hybrid, and storage artifacts as the primary sources, and do not broaden scope into GPU work unless the reassessment exposes a concrete dependency.
- Preserve the hard-cutover policy: do not add compatibility bridges or parallel orchestration contracts just to keep multiple Kubernetes shapes alive.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when VDcStSMlp has been converted into an explicit next board state and only manual verification remains
- YIELD to human when prioritization depends on provider choice, control-plane topology, or external cluster commitments that are not inferable from the repository

# Record Port Product Horizon - Charter

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Capture first-class Linux and Mac developer experience as a board-managed horizon bearing. | board: VDcT0vaPb |
| MG-02 | Capture hybrid local, remote, cloud, and SSH-first execution as a board-managed horizon bearing. | board: VDcStPolu |
| MG-03 | Capture first-class k3s and Kubernetes workload support as a board-managed horizon bearing. | board: VDcStSMlp |
| MG-04 | Capture cloud block-storage normalization as a board-managed horizon bearing. | board: VDcStQqlo |
| MG-05 | Capture GPU execution support as a board-managed horizon bearing. | board: VDcStPNlr |
| MG-06 | Leave the board doctor-clean with the new horizon visible through mission, bearing-list, and flow surfaces. | manual: run `just keel doctor`, `just keel bearing list`, and `just flow` |

## Constraints

- Capture these themes as bearings, not premature epics or stories.
- Keep the recorded horizon aligned with Port's current product shape: one canonical CLI, one guest protocol, and explicit local versus hosted ownership.
- Use existing repo artifacts as the primary research basis and incorporate the referenced Slicer k3s material where it sharpens the next research slice.
- Preserve clean board generation after adding the new horizon items.

## Halting Rules

- DO NOT halt while any requested horizon area lacks a recorded bearing.
- HALT when all requested horizon areas are captured and only manual verification remains.
- YIELD to human when prioritization depends on budget, vendor commitments, or external infrastructure choices that are not inferable from the repository.

# Ship Cloud Block Storage Normalization - Charter

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Convert bearing VDcStQqlo into executable board work for a canonical cloud block-storage contract across local, hosted, and SSH-owned execution lanes. | board: VDcStQqlo |
| MG-02 | Keep the mission surfaces doctor-clean and make this cloud block-storage mission visible through the board command surfaces. | manual: run `just keel mission show VDfCa3q94`, `just flow`, and `just keel doctor` |

## Constraints

- Keep the first slice bounded to storage normalization: volume identity, attachment semantics, placement or ownership language, and one proof-backed operator workflow. Do not broaden scope into a full storage service, CSI-style orchestration, k3s, or GPU work.
- Use bearing VDcStQqlo plus the current artifact, rootfs, hosted-placement, and hybrid-execution artifacts as the primary sources, and preserve one canonical `port` operator surface rather than inventing a second storage-specific control plane.
- Preserve the hard-cutover policy: do not add compatibility aliases, dual storage models, or fallback parsing to bridge old artifact-only semantics with the new canonical storage contract.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when the storage-normalization work is decomposed into executable board items and only verification-oriented manual checks remain
- YIELD to human when prioritization depends on provider-specific persistence guarantees, block-device API choices, or durability semantics that are not inferable from the repository

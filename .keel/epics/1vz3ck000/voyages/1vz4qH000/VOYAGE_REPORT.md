# VOYAGE REPORT: Mac Operator Shell Compatibility

## Voyage Metadata
- **ID:** 1vz4qH000
- **Epic:** 1vz3ck000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 1/1 stories complete

## Implementation Narrative
### Gate Linux Only Dev Shell Inputs
- **ID:** 1vz4rP000
- **Status:** done

#### Summary
Keep `nix develop` usable on macOS by gating Linux-only runtime packages in the
default dev shell while preserving the Linux toolchain Port still needs for
local Firecracker launch.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `flake.nix` no longer attempts to evaluate unsupported Linux-only runtime packages on macOS, and Darwin shell evaluation succeeds without unsupported-system overrides. <!-- [SRS-01/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz4rP000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-02/AC-01] The Linux shell still includes Firecracker and Linux networking/runtime tools required by Port's local launch workflow. <!-- [SRS-02/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz4rP000/verify-ac-2.sh, proof: ac-2.log -->
- [x] [SRS-03/AC-01] Docs or shell messaging explain that the macOS shell is for repo tooling while Linux-only runtime tools remain omitted. <!-- [SRS-03/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz4rP000/verify-ac-3.sh, proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vz4rP000/EVIDENCE/ac-1.log)
- [ac-3.log](../../../../stories/1vz4rP000/EVIDENCE/ac-3.log)
- [ac-2.log](../../../../stories/1vz4rP000/EVIDENCE/ac-2.log)



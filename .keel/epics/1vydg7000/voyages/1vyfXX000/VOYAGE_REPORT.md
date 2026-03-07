# VOYAGE REPORT: Remove Nix Bias From Help Surface

## Voyage Metadata
- **ID:** 1vyfXX000
- **Epic:** 1vydg7000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 1/1 stories complete

## Implementation Narrative
### Make Help Examples Runtime Agnostic
- **ID:** 1vyfXg000
- **Status:** done

#### Summary
Remove the nix-specific prescription from the canonical help/examples and align
the supporting docs on generic runtime prerequisites like required tools on
`PATH` and `port doctor`.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `port --help` describes local example prerequisites without prescribing `nix develop`. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && /tmp/port-target/debug/port --help | rg -n "PATH|port doctor|repository root" && ! /tmp/port-target/debug/port --help | rg -q "nix develop"', proof: ac-1.log-->
- [x] [SRS-02/AC-01] README and operator docs explain the same generic prerequisite boundary without treating Nix as required runtime behavior. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "PATH|port doctor|repository root" /home/alex/workspace/spoke-sh/port/README.md /home/alex/workspace/spoke-sh/port/docs/operators.md && ! rg -q "nix develop" /home/alex/workspace/spoke-sh/port/README.md /home/alex/workspace/spoke-sh/port/docs/operators.md', proof: ac-2.log-->
- [x] [SRS-03/AC-01] Recorded evidence shows the updated help surface directs operators to tool availability and `port doctor` rather than Nix. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && /tmp/port-target/debug/port --help | head -n 80', proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vyfXg000/EVIDENCE/ac-1.log)
- [ac-3.log](../../../../stories/1vyfXg000/EVIDENCE/ac-3.log)
- [ac-2.log](../../../../stories/1vyfXg000/EVIDENCE/ac-2.log)



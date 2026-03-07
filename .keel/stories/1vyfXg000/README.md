---
id: 1vyfXg000
title: Make Help Examples Runtime Agnostic
type: feat
status: backlog
created_at: 2026-03-06T16:29:16
updated_at: 2026-03-06T16:30:22
scope: 1vydg7000/1vyfXX000
---

# Make Help Examples Runtime Agnostic

## Summary

Remove the nix-specific prescription from the canonical help/examples and align
the supporting docs on generic runtime prerequisites like required tools on
`PATH` and `port doctor`.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [ ] [SRS-01/AC-01] `port --help` describes local example prerequisites without prescribing `nix develop`. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && /tmp/port-target/debug/port --help | rg -n "PATH|port doctor|repository root" && ! /tmp/port-target/debug/port --help | rg -q "nix develop"', proof: ac-1.log-->
<!-- verify: manual, SRS-02:start:end, proof: ac-2.log-->
- [ ] [SRS-02/AC-01] README and operator docs explain the same generic prerequisite boundary without treating Nix as required runtime behavior. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "PATH|port doctor|repository root" /home/alex/workspace/spoke-sh/port/README.md /home/alex/workspace/spoke-sh/port/docs/operators.md && ! rg -q "nix develop" /home/alex/workspace/spoke-sh/port/README.md /home/alex/workspace/spoke-sh/port/docs/operators.md', proof: ac-2.log-->
<!-- verify: manual, SRS-03:start:end, proof: ac-3.log-->
- [ ] [SRS-03/AC-01] Recorded evidence shows the updated help surface directs operators to tool availability and `port doctor` rather than Nix. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && /tmp/port-target/debug/port --help | sed -n "1,80p"', proof: ac-3.log-->

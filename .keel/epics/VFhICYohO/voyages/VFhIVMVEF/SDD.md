# Foundational AWS PVM Docs Refresh - Software Design Description

> Converge the foundational docs and public AWS narrative on one clear x86_64 hosted PVM production contract.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage is documentation-only. It improves fidelity by replacing the
current split AWS narrative with one deliberate hierarchy:

1. Root docs explain where Port is strongest today and point readers to the
   canonical AWS/PVM contract.
2. Focused docs carry the operational depth for hosted AWS PVM, including host
   kit, artifact kit, preparation, launch, and failure surfaces.
3. Public docs mirror the same posture in a simpler production narrative
   instead of treating AWS PVM as a side note to the standard hosted lane.

## Context & Boundaries

In scope is documentation structure and wording across root docs, focused
guides, and the Docusaurus AWS narrative. Out of scope are runtime changes,
provider automation, or any expansion of the shipped support matrix.

```
┌─────────────────────────────────────────────────────────────┐
│                    Foundational Docs Refresh                │
│                                                             │
│  README / ARCHITECTURE / CONFIGURATION                      │
│                 ↓                                           │
│       hosted.md / cloud.md / pvm.md                         │
│                 ↓                                           │
│      website/docs/path-to-production/aws.mdx                │
└─────────────────────────────────────────────────────────────┘
             ↑                           ↑
      examples/port.toml          verified AWS PVM contract
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `examples/port.toml` | local source | Keeps config examples and lane names aligned with the actual shipped model | repo head |
| Verified mission `VFgcM1Zpu` | board artifact | Supplies the current AWS hosted PVM runtime truth that docs should reflect | `.keel/missions/VFgcM1Zpu` |
| Public docs site | local source | Keeps the Docusaurus AWS narrative aligned with root docs | `website/` |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Canonical production narrative | Make AWS x86_64 hosted Firecracker/PVM the strongest production-oriented cloud path | Matches the real product gap the user called out and the verified hosted AWS PVM mission |
| Simplification strategy | Add or strengthen one canonical AWS/PVM path and make adjacent docs link to it instead of restating everything | Reduces duplication instead of adding another overlapping guide |
| Truthfulness boundary | Keep standard hosted, GCP/Azure, repo-local proof harnesses, and arm64 PVM clearly subordinate or out of scope | Higher fidelity depends on explicit boundaries, not marketing language |

## Architecture

The documentation architecture should mirror the product hierarchy:

- root docs answer "what is Port strongest at today?"
- focused docs answer "how does AWS hosted PVM actually work?"
- public docs answer "what is the clearest production path for a new reader?"

## Components

- `README.md`: concise product posture, simplified doc map, and strongest path
  call-out
- `ARCHITECTURE.md`: lane hierarchy and ownership model, with AWS hosted PVM
  treated as the clearest production-oriented hosted lane
- `CONFIGURATION.md`: config shape and workflow examples for the AWS hosted PVM
  path
- `docs/hosted.md`, `docs/cloud.md`, `docs/pvm.md`: operational contract and
  boundaries
- `website/docs/path-to-production/aws.mdx`: public production narrative

## Interfaces

This voyage does not introduce a new product interface. It only changes how
existing documented contracts are explained and cross-linked.

## Data Flow

Source of truth should flow from the verified runtime contract and checked-in
config model into the focused docs, then upward into README and public docs.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Docs overstate support | Manual review finds implied claims beyond AWS x86_64 hosted PVM | Rewrite the section to restore explicit boundaries | Recheck against `examples/port.toml` and verified AWS PVM mission artifacts |
| Docs remain duplicated or contradictory | Cross-reading root and focused docs reveals multiple "primary" AWS stories | Consolidate onto one canonical section and replace repetition with links | Review README plus hosted/cloud/PVM docs together before closing |

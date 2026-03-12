# Installable Linux And Mac Developer Experience — Brief

## Hypothesis

Port will become reusable across many projects much faster if Linux and Mac
operators can install a supported `port` binary with a clear target matrix,
checks, and update story instead of treating source checkout plus repo fluency
as the default product contract.

## Problem Space

Port already ships real Linux and macOS lanes, but the current release contract
still leaves packaged binaries, installers, signatures, automated publication,
and a stricter target matrix as follow-on work. That keeps Port usable for the
current repository while slowing adoption in external projects.

## Context

The repo now has:

- Linux as the primary local Firecracker and hosted proof lane,
- macOS as the AVF local lane plus repo tooling lane,
- and a `just mission` plus `just doctor` release-validation surface.

The user wants to start using Port in many other projects. That moves the next
gap from runtime breadth to product distribution and operator experience.

## Objectives

- Define the first-class installation contract for Linux and Mac users.
- Tighten the support matrix from broad platform labels into explicit release
  targets and expectations.
- Sequence the minimum packaging work needed for the AVF lane to feel
  installable rather than repo-local.
- Keep `port doctor`, CLI help, and release docs aligned with the product
  contract.

## Scope

- In scope: packaged binaries, installers, checksums or signatures, release
  publication expectations, support-matrix refinement, and macOS AVF packaging
  boundaries.
- Out of scope: a Windows-native local runtime lane, App Store distribution,
  or a redesign of the Linux and AVF runtime model.

## Success Criteria

- [ ] A concrete Linux and Mac release target matrix is defined with operator
  expectations by target triple.
- [ ] The first packaging and installer contract is explicit enough to drive an
  epic instead of another vague release wishlist.
- [ ] The macOS AVF helper, entitlement, and signing boundary is captured as an
  explicit part of the product story.
- [ ] Release validation remains anchored on the canonical `port` and `just`
  surfaces instead of source-only workflows.

## Research Questions

- Which installation paths should be first-class for Linux and Mac operators?
- What is the smallest support matrix that still feels product-grade?
- How should the AVF helper and entitlement boundary ship on macOS?
- What release proof should be human-readable enough for operators evaluating
  Port outside this repo?

## Open Questions

- Should Port ship one universal tarball per target, native installers, or
  both?
- How much update and signing machinery belongs in the first release slice
  versus later hardening?
- Does the first Mac product surface ship only CLI plus helper, or also a more
  opinionated launcher or wrapper?

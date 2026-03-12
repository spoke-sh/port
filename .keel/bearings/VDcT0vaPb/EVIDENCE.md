---
id: VDcT0vaPb
---

# Installable Linux And Mac Developer Experience — Evidence

## Sources

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | manual | manual:doc-review | /home/alex/workspace/spoke-sh/port/RELEASE.md | 2026-03-11 | 2026-03-11 | high | high | The current release contract explicitly leaves installers, signatures, publication, and a tighter target matrix as follow-on work. |
| SRC-02 | manual | manual:doc-review | /home/alex/workspace/spoke-sh/port/README.md | 2026-03-11 | 2026-03-11 | high | high | README publishes the current Linux, macOS, and Windows support boundary that future packaging must respect. |
| SRC-03 | manual | manual:doc-review | /home/alex/workspace/spoke-sh/port/docs/avf.md | 2026-03-11 | 2026-03-11 | high | high | The AVF contract shows that macOS already has a real lane, but still carries helper and entitlement packaging boundaries. |

## Feasibility

Feasible. Port already has a canonical CLI surface, a published release
checklist, and a first-class macOS substrate contract. The missing work is
distribution and support normalization, not a fresh runtime architecture.

## Findings

### 1. Productization is now the main adoption gap

`RELEASE.md` explicitly calls out packaged binaries, installers, checksums or
signatures, automated release publication, and a tighter support matrix as open
work. That means Port's next adoption barrier is not feature absence alone; it
is the lack of an installable product surface [SRC-01].

### 2. Linux and Mac already justify a real release matrix

README now publishes Linux as the primary local and hosted lane and macOS as
the AVF local lane. That is enough breadth to justify a first-class release
contract instead of leaving platform support implicit or source-only [SRC-02].

### 3. macOS needs packaging, not just shell compatibility

The AVF contract already describes a real operator workflow, but it still
depends on a launcher helper plus Apple's entitlement boundary. A first-class
Mac experience therefore needs packaging, helper discovery, and signing
decisions, not just `nix develop` compatibility [SRC-03].

## Open Technical Risks

- Linux packaging can overpromise runtime prerequisites if installers blur the
  line between the `port` binary and host-level hypervisor requirements.
- macOS packaging may fragment if the helper, entitlement, and CLI distribution
  paths are treated as separate products.
- Release proofs can remain too low-level unless they surface install success,
  version identity, and one human-readable launch or guest workflow.

## Key Findings

1. Port's next adoption gap is product distribution rather than missing core
   CLI shape [SRC-01].
2. Linux and macOS already justify a clearer target matrix and release contract
   [SRC-02].
3. AVF makes macOS a real product lane, but only if packaging captures the
   helper and entitlement boundary [SRC-03].

## Unknowns

- Which packaging channel should be canonical first: tarballs, Homebrew, Nix
  artifacts, native installers, or a small combination?
- How much signing and update automation is required before the first
  cross-project release feels credible?

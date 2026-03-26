# Release Process

Port is still in an early product phase, but the release contract should stay
explicit.

## Current State

Port currently ships as a Rust workspace with:

- a CLI binary surface in `port-cli`
- a shared model/runtime split
- proof-backed local and hosted workflow slices
- board-managed verification evidence for major delivery work

Distribution automation is still follow-on work. The release contract today is
mostly about versioning, validation, package boundaries, and support surfaces.

## Supported Install Targets

| Target triple | Current role | Boundary |
|---------------|--------------|----------|
| `x86_64-unknown-linux-gnu` | Primary CLI package for local Firecracker workflows and hosted proof lanes on Linux. | Firecracker still requires a Linux host; PVM remains a prepared-node x86_64 lane rather than a generic local fallback. |
| `x86_64-apple-darwin` | Intel macOS CLI package for the AVF local lane. | AVF requires a local macOS host plus an external launcher helper set through `PORT_AVF_LAUNCHER`. |
| `aarch64-apple-darwin` | Apple Silicon macOS CLI package for the AVF local lane. | Distributed targets still need Apple's virtualization entitlement and related sandbox entitlements when applicable. |
| Windows | No native install package in this slice. | Use WSL or a remote Linux host for Linux-backed workflows. |

## First Package Contract

The first installable Port package is a versioned tarball per supported target:

- `port-<version>-<target-triple>.tar.gz`

The package contract for this slice is:

- ship the canonical `port` CLI binary
- include release metadata and install guidance alongside the binary
- keep `port doctor` as the first post-install verification step
- keep macOS AVF on the canonical `port` CLI plus an external
  `PORT_AVF_LAUNCHER` helper rather than a bundled macOS-only launcher
  workflow
- leave native installers, Homebrew taps, and automated publication as
  follow-on work

## Release Checklist

1. Update the crate version metadata that should move with the release.
2. Review release notes, docs, and support boundaries, including
   `docs/install.md` and `docs/avf.md`.
3. Run the canonical validation path:

```bash
keel mission show <mission-id>
just test
just doctest
just package x86_64-unknown-linux-gnu
just package-proof x86_64-unknown-linux-gnu
```

4. Confirm the packaged verification entrypoint still works from the installed
   proof prefix:

```bash
artifacts/package-proof/x86_64-unknown-linux-gnu/prefix/bin/port doctor
```

5. Confirm the board is clean:

```bash
keel doctor
```

6. Commit the release metadata and tag the revision.

## Validation Expectations

Release validation should confirm:

- the workspace tests pass
- doctests pass
- the canonical package and package-proof commands pass
- the packaged `port doctor` check remains the post-install gate
- the board is doctor-clean
- the current mission signal is legible through `keel mission show <mission-id>`
- top-level docs and help surfaces match shipped behavior

## Open Release Work

- packaged binaries and installers
- checksums or signatures
- automated release publication
- a stricter support matrix by target triple

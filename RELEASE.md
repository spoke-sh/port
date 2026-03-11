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
mostly about versioning, validation, and support boundaries.

## Supported Targets

| Target | Current role |
|--------|--------------|
| Linux | Primary local Firecracker and hosted proof lane |
| macOS | AVF local lane plus repo tooling |
| Windows | Linux-backed operator workflow through WSL or a remote Linux host |

## Release Checklist

1. Update the crate version metadata that should move with the release.
2. Review release notes, docs, and support boundaries.
3. Run the validation path:

```bash
just mission
```

4. Confirm the board is clean:

```bash
just keel doctor
```

5. Commit the release metadata and tag the revision.

## Validation Expectations

Release validation should confirm:

- the workspace tests pass
- doctests pass
- the board is doctor-clean
- the current mission signal is legible through `just mission`
- top-level docs and help surfaces match shipped behavior

## Open Release Work

- packaged binaries and installers
- checksums or signatures
- automated release publication
- a stricter support matrix by target triple

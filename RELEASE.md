# Release Process

This project uses [cargo-dist](https://opensource.axo.dev/cargo-dist/) to
publish release archives and installers when a version tag is pushed. Port
keeps the supported target matrix explicit and ships one canonical upgrade
surface through `port upgrade`.

## Supported Release Targets

| Target triple | Release role | Boundary |
|---------------|--------------|----------|
| `x86_64-unknown-linux-gnu` | Primary release target for local Firecracker workflows and hosted proof lanes on Linux | Firecracker still requires a Linux host; PVM remains a prepared-node `x86_64` lane rather than a generic fallback |
| `x86_64-apple-darwin` | Intel macOS release target for the AVF local lane | AVF still requires an external `PORT_AVF_LAUNCHER` helper |
| `aarch64-apple-darwin` | Apple Silicon macOS release target for the AVF local lane | Distributed targets remain bounded by Apple's virtualization entitlement requirements |
| Windows | No native release target in this slice | Use WSL or a remote Linux host for Linux-backed workflows |

## Install And Upgrade Contract

Latest release install:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/spoke-sh/port/releases/latest/download/port-installer.sh | sh
port doctor
```

Latest release upgrade:

```bash
port upgrade
```

Source-built revision install:

```bash
port upgrade --tag <tag>
port upgrade --sha <git-sha>
```

The source-install flow reuses a cached checkout under `~/.cache/port`, builds
with a supported local Rust toolchain, and installs the built binary plus the
bundled runtime assets into the standard Cargo-style prefix.

## Release Workflow

Pushing a tag that starts with `v` or otherwise matches the repository's
version-tag pattern triggers [.github/workflows/release.yml](.github/workflows/release.yml).
The workflow will:

- run `dist plan` on pull requests and `dist host --steps=create` on tagged pushes
- build release archives and the shell installer for Port's supported targets
- publish the resulting release artifacts to GitHub Releases
- update a Homebrew tap when `HOMEBREW_TAP_REPO` and `HOMEBREW_TAP_TOKEN` are configured

## Release Checklist

1. Update the workspace version in `Cargo.toml`.
2. Review release notes and support boundaries in `docs/install.md` and `docs/avf.md`.
3. Run the canonical validation path:

```bash
keel mission show <mission-id>
just test
just doctest
just package-proof x86_64-unknown-linux-gnu
dist plan
```

4. Confirm the packaged verification entrypoint still works from the installed proof prefix:

```bash
artifacts/package-proof/x86_64-unknown-linux-gnu/prefix/bin/port doctor
```

5. Confirm the board is clean:

```bash
keel doctor
```

6. Commit the release metadata, push the change, create a version tag, and push the tag.

## Validation Expectations

Release validation should confirm:

- the workspace tests pass
- doctests pass
- the repo-local package proof still passes
- `dist plan` succeeds with the checked-in cargo-dist config
- the packaged `port doctor` check remains the post-install gate
- the board is doctor-clean
- top-level docs and help surfaces match shipped behavior

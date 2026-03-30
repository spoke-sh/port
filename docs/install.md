# Installable Port Contract

Port's first installable release slice is intentionally small: ship the
canonical `port` CLI as a versioned target-specific tarball, keep one operator
vocabulary across Linux and macOS, and make unsupported environments explicit.

## Canonical Package Artifact

The first installable Port artifact is a versioned tarball per supported target
triple:

- `port-<version>-<target-triple>.tar.gz`

The package contract for this slice is:

- ship the canonical `port` CLI binary
- include release metadata and install guidance alongside the binary
- keep `port doctor` as the first post-install verification step
- avoid native installers, app bundles, Homebrew taps, or a second
  platform-specific command family in this slice

## Supported Install Targets

| Target triple | Supported path | Boundary |
|---------------|----------------|----------|
| `x86_64-unknown-linux-gnu` | Primary CLI package for local Firecracker workflows and hosted control-plane demos on Linux. | Local Firecracker still requires a Linux host; Firecracker/PVM remains a dedicated prepared-node lane rather than a generic fallback. |
| `x86_64-apple-darwin` | macOS CLI package for the AVF local lane on Intel Macs. | AVF remains a local macOS workflow and still requires an external launcher helper via `PORT_AVF_LAUNCHER`. |
| `aarch64-apple-darwin` | macOS CLI package for the AVF local lane on Apple Silicon Macs. | Distributed targets still need Apple's virtualization entitlement; Rosetta convenience is not part of the core package contract. |

## Unsupported Environments In This Slice

| Environment | Current boundary |
|-------------|------------------|
| Windows native install | Not shipped in the first installable slice. Use WSL or a remote Linux host for Linux-backed workflows. |
| Linux targets outside the published matrix | Not part of the first package contract until Port publishes a supported target triple and package proof for them. |
| Bundled AVF launcher app | Not shipped here. AVF uses the canonical `port` CLI plus an explicitly configured launcher helper. |

## Operator Verification Path

Once a supported package is installed, the canonical verification entrypoint is:

```bash
port doctor
```

That confirms the host and lane boundary without requiring repo-local Cargo
commands.

### Self-Contained Package Prefix

The `port` CLI package is self-contained. It includes:

- The `port` binary itself
- Required runtime assets (scripts, examples, bootstrap kits) under `share/port`
- Wrapped execution with dependencies (Firecracker, K3s, ORAS, etc.) on its `PATH`

This allows `port` to be invoked reproducibly in CI, AWS, and SRE environments
without a repository checkout. Relative paths in the sample configuration
automatically resolve to the package's bundled assets.

# Installable Port Contract

Port publishes installable release artifacts through cargo-dist while keeping
the support matrix explicit. Linux and macOS use the same canonical `port`
command surface, and Windows remains a workstation path through WSL or a remote
Linux host.

## Install The Latest Release

Use the published shell installer:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/spoke-sh/port/releases/latest/download/port-installer.sh | sh
port doctor
```

That is the fastest path to a released CLI. The first verification step stays
the same everywhere:

```bash
port doctor
```

## Upgrade Paths

Upgrade to the latest release:

```bash
port upgrade
```

Install a pinned revision from source:

```bash
port upgrade --tag <tag>
port upgrade --sha <git-sha>
```

The revision-pinned path reuses a cached checkout under `~/.cache/port`, builds
with a supported local Rust toolchain, and installs the result into the same
Cargo-style prefix used by the release installer.

## Supported Install Targets

| Target triple | Supported path | Boundary |
|---------------|----------------|----------|
| `x86_64-unknown-linux-gnu` | Primary release target for local Firecracker workflows and hosted control-plane demos on Linux | Local Firecracker still requires a Linux host; Firecracker/PVM remains a dedicated prepared-node lane rather than a generic fallback |
| `x86_64-apple-darwin` | Intel macOS release target for the AVF local lane | AVF remains a local macOS workflow and still requires an external launcher helper via `PORT_AVF_LAUNCHER` |
| `aarch64-apple-darwin` | Apple Silicon macOS release target for the AVF local lane | Distributed targets still need Apple's virtualization entitlement; Rosetta convenience is not part of the core package contract |

## Unsupported Environments In This Slice

| Environment | Current boundary |
|-------------|------------------|
| Windows native install | Not shipped in the first cargo-dist release slice. Use WSL or a remote Linux host for Linux-backed workflows |
| Linux targets outside the published matrix | Not part of the install contract until Port publishes and validates that target |
| Bundled AVF launcher app | Not shipped here. AVF uses the canonical `port` CLI plus an explicitly configured launcher helper |

## Bundled Runtime Assets

Released Port installs remain self-contained. They include:

- the `port` binary
- bundled docs, examples, and artifact scripts needed by supported workflows
- the install metadata required for `port upgrade` and post-install verification

Port resolves both the legacy packaged `share/port/...` layout and the
cargo-dist installer layout so released binaries can still find the assets they
need without a repository checkout.

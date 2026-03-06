# Operator Workflows

This document explains the supported MVP workflows for Linux, macOS, and
Windows operators.

## Support Matrix

| Operator environment | Supported MVP workflow | Unsupported MVP workflow | Why |
|----------------------|------------------------|--------------------------|-----|
| Linux host with `/dev/kvm` and Firecracker | Run the full local Port workflow directly through the `port` CLI | n/a | Firecracker local launch requires Linux and KVM |
| macOS workstation | Edit or inspect the repo locally, then run `port` on a Linux host | Local Firecracker launch on macOS | Firecracker local launch requires Linux and `/dev/kvm` |
| Windows workstation | Use WSL or a remote Linux host for the Linux-side `port` workflow | Native Windows Firecracker launch | Firecracker local launch requires a Linux environment with `/dev/kvm`; not every WSL setup exposes that capability |

## Linux Workflow

Use this workflow when `port doctor` passes on a Linux host.

```bash
nix develop
cargo run -p port-cli -- --config examples/port.toml artifacts build --artifact demo-kernel
cargo run -p port-cli -- --config examples/port.toml artifacts validate --artifact demo-kernel
cargo run -p port-cli -- --config examples/port.toml artifacts build --artifact demo-guest
cargo run -p port-cli -- --config examples/port.toml artifacts validate --artifact demo-guest
cargo run -p port-cli -- --config examples/port.toml doctor
cargo run -p port-cli -- --config examples/port.toml machine launch --machine demo
```

Artifacts land under `artifacts/`. Runtime manifests and console logs land under
the chosen runtime root, which defaults to `runtime/`.

Current guest-command behavior:

- `port guest exec`, `copy`, `pty`, `logs`, and `forward` use the runtime
  guest-agent socket at `<runtime-root>/<machine>/guest-agent.sock`.
- The built guest image carries `port-guest-agent`, but the launched-guest
  transport is not yet the canonical CLI path.

## macOS Workflow

The supported macOS workflow is to run the actual Port commands on a Linux host.

Recommended path:

1. Keep the repository on the Linux host directly, or SSH into that host from macOS.
2. On the Linux host, run the same canonical commands shown in the Linux workflow.
3. Use `port doctor` on that Linux host before attempting local Firecracker launch.

Unsupported path:

- Running local Firecracker launch directly on macOS is unsupported because the
  MVP launch path requires Linux and `/dev/kvm`.

## Windows Workflow

The supported Windows workflow is a Linux-backed one:

1. Use WSL if you want a local Linux shell for the repository and CLI.
2. Run `port doctor` inside that Linux environment.
3. If `port doctor` reports missing `/dev/kvm`, Firecracker, or other required
   Linux host capabilities, switch to a remote Linux host and run the same `port`
   commands there.

Supported path:

- WSL or remote Linux is valid for editing the model, building artifacts, and
  running the canonical CLI.

Current constraint:

- Native Windows is not a supported Firecracker execution environment for the
  MVP, and WSL availability of the required Linux virtualization features is an
  environment check rather than a Port guarantee.

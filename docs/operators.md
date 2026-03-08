# Operator Workflows

This document explains the supported MVP workflows for Linux, macOS, and
Windows operators.

## Support Matrix

| Operator environment | Supported MVP workflow | Unsupported MVP workflow | Why |
|----------------------|------------------------|--------------------------|-----|
| Linux host with `/dev/kvm` and Firecracker | Run the full local Port workflow directly through the `port` CLI | n/a | Firecracker local launch requires Linux and KVM |
| macOS workstation | Edit or inspect the repo locally, then run `port` on a Linux host today; AVF is the first-class planned macOS lane | Local Firecracker launch on macOS | Firecracker local launch requires Linux and `/dev/kvm`; AVF is planned but not yet shipped |
| Windows workstation | Use WSL or a remote Linux host for the Linux-side `port` workflow | Native Windows Firecracker launch | Firecracker local launch requires a Linux environment with `/dev/kvm`; not every WSL setup exposes that capability |

## Linux Workflow

Use this workflow when `port doctor` passes on a Linux host.
Run the sample-config commands from the repository root so
`examples/port.toml` resolves correctly.

```bash
cargo run -p port-cli -- doctor
cargo run -p port-cli -- --config examples/port.toml artifacts build --artifact demo-kernel
cargo run -p port-cli -- --config examples/port.toml artifacts validate --artifact demo-kernel
cargo run -p port-cli -- --config examples/port.toml artifacts build --artifact demo-guest
cargo run -p port-cli -- --config examples/port.toml artifacts validate --artifact demo-guest
cargo run -p port-cli -- --config examples/port.toml doctor
cargo run -p port-cli -- --config examples/port.toml machine launch --machine demo
cargo run -p port-cli -- machine list
cargo run -p port-cli -- machine status --machine demo
cargo run -p port-cli -- machine stop --machine demo
```

Artifacts land under `artifacts/`. Runtime manifests and console logs land under
the chosen runtime root, which defaults to `runtime/`.

Important prerequisite note:

- The sample artifact and launch workflow assumes `firecracker`,
  artifact-build tools, and the Linux networking utilities that `port doctor`
  checks are available in the execution environment.
- If `port doctor` reports a missing dependency, treat that as the explanation
  for why a later `port machine launch` example will fail.

Current lifecycle behavior:

- `port machine list` enumerates Port-managed runtime directories and reports
  `running`, `stopped`, `stale`, or `malformed` state from manifests plus live
  PID inspection.
- After a machine has been launched, `list`, `status`, and `stop` operate on
  the runtime root directly and do not require the model file again.
- `port machine status --machine demo` prints the runtime directory, config
  path, manifest, pid file, and console/log references needed for debugging.
- `port machine stop --machine demo` signals a live Firecracker process and
  cleans stale pid/vsock/socket files so the next launch is deterministic.
- Those commands are the local ownership implementation of Port's longer-term
  hosted contract. In hosted Port, the CLI keeps the same verbs while a
  node-local agent plus control plane take over runtime ownership.

Current guest-command behavior:

- `port guest exec`, `copy`, `pty`, `logs`, and `forward` now work against
  launched Firecracker VMs through the machine model's live guest control port.
- `port guest copy` transfers bytes across the real host/guest boundary; it no
  longer depends on the guest seeing host paths directly.
- `port guest forward` binds on the host and stays attached in the foreground
  until you stop it.
- Guest-side `port guest forward --target ...` addresses still depend on guest
  networking being up. In the sample guest image, bring loopback up before
  targeting `127.0.0.1`, for example with
  `port guest exec --machine demo -- /bin/sh -lc 'busybox ifconfig lo up'`.

## Remote Linux And Cloud Workflow

The shared model now includes explicit provider identity for remote Linux and
cloud-adjacent hosts:

- `generic-linux` for a future remote Linux control lane
- `aws` and `gcp` for the currently justified future cloud lanes
- `azure` for an explicitly unsupported Firecracker MVP lane

Use the canonical CLI to inspect that boundary:

```bash
cargo run -p port-cli -- --config examples/port.toml doctor
cargo run -p port-cli -- --config examples/port.toml machine launch --machine cloud-aws
```

What to expect:

- `port doctor` shows provider-aware checks for `generic-linux`, `aws`, `gcp`,
  and `azure` alongside the usual local Linux prerequisites.
- `port machine launch --machine cloud-aws` is expected to fail with AWS-
  specific guidance because remote launch orchestration is not implemented in
  the MVP.
- `port machine launch --machine demo` remains the supported local Linux launch
  proof for the MVP.
- `port machine list`, `status`, and `stop` currently inspect only local
  runtime roots; they do not yet enumerate or control remote/cloud hosts.
- The future hosted split for lifecycle ownership and guest-operation brokering
  is documented in [`hosted.md`](hosted.md).

The full cloud matrix and substrate lane guidance live in [`docs/cloud.md`](cloud.md).
The hosted node-agent/control-plane split that will eventually sit behind the
same CLI lives in [`docs/hosted.md`](hosted.md).

## macOS Workflow

The shipped macOS workflow is still to run the actual Firecracker commands on a
Linux host.

Recommended path:

1. Keep the repository on the Linux host directly, or SSH into that host from macOS.
2. On the Linux host, run the same canonical commands shown in the Linux workflow.
3. Use `port doctor` on that Linux host before attempting local Firecracker launch.

Current boundary:

- Running local Firecracker launch directly on macOS is unsupported because the
  MVP launch path requires Linux and `/dev/kvm`.
- Apple Virtualization Framework is now a first-class planned Port lane, but it
  is not executable in the current runtime yet.
- Remote cloud hosts should still be treated as Linux execution environments:
  run `port doctor` and any future launch commands on the Linux side, not on
  macOS itself.

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
- The cloud provider matrix is still a Linux-hosted workflow; Windows changes
  the operator workstation, not the underlying Firecracker requirement.

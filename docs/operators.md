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
cargo run -p port-cli -- --config examples/port.toml artifacts build --artifact demo-kernel --architecture native
cargo run -p port-cli -- --config examples/port.toml artifacts validate --artifact demo-kernel --architecture native
cargo run -p port-cli -- --config examples/port.toml artifacts build --artifact demo-guest --architecture native
cargo run -p port-cli -- --config examples/port.toml artifacts validate --artifact demo-guest --architecture native
cargo run -p port-cli -- --config examples/port.toml doctor
cargo run -p port-cli -- --config examples/port.toml machine launch --machine demo
cargo run -p port-cli -- machine list
cargo run -p port-cli -- machine status --machine demo
cargo run -p port-cli -- machine stop --machine demo
```

Artifact variants land under
`artifacts/<kind>/<name>/<architecture>/<substrate>/<protection-mode>/`.
Runtime manifests and console logs land under the chosen runtime root, which
defaults to `runtime/`.

Artifact mobility quick reference:

```bash
cargo run -p port-cli -- --config examples/port.toml artifacts push --artifact demo-kernel --architecture x86-64
rm -f artifacts/kernel/demo/x86_64/firecracker/standard/vmlinux
cargo run -p port-cli -- --config examples/port.toml artifacts pull --artifact demo-kernel --architecture x86-64
```

Those commands use the artifact's configured mobility backend. In the sample
config, that means a file-backed store at `artifact-store/demo-fs/` plus a
local cache at `.port/cache/`.

The local lifecycle commands are the canonical way to manage a launched Port
machine:

- `port machine list` enumerates Port-managed runtime directories under the
  local runtime root and gives you the inventory view for locally owned
  machines.
- `port machine status --machine <name>` reads one machine's local runtime
  state back out of that runtime root so you can inspect manifests, pid files,
  config paths, and console/log paths without talking to Firecracker directly.
- `port machine stop --machine <name>` stops a Port-managed local machine and
  clears runtime ownership details that would otherwise make a relaunch
  ambiguous.

Important prerequisite note:

- The sample artifact and launch workflow assumes `firecracker`,
  artifact-build tools, and the Linux networking utilities that `port doctor`
  checks are available in the execution environment.
- `nix develop` is one way to provide those tools, but it is not a Port
  runtime requirement. Installing the required tools on the host directly is
  equally valid.
- If `port doctor` reports a missing dependency, treat that as the explanation
  for why a later `port machine launch` example will fail.

Current lifecycle behavior:

- `port machine list` enumerates Port-managed runtime directories and reports
  `running`, `stopped`, `stale`, or `malformed` state from manifests plus live
  PID inspection.
- `port machine list`, `status`, and `stop` also publish the local control
  contract: `inventory scope`, `inventory owner`, `lifecycle owner`, `status
  source`, and routing fields that currently resolve to the local runtime root.
- After a machine has been launched, `list`, `status`, and `stop` operate on
  the runtime root directly and do not require the model file again.
- `port machine status --machine demo` prints the runtime directory, config
  path, manifest, pid file, console/log references, and the control-contract
  routing fields needed for debugging or for mapping the same verbs onto future
  hosted ownership.
- `port machine stop --machine demo` signals a live Firecracker process and
  cleans stale pid/vsock/socket files so the next launch is deterministic.
- Those commands are the local ownership implementation of Port's longer-term
  hosted contract. In hosted Port, the CLI keeps the same verbs while a
  node-local agent plus control plane take over runtime ownership.

Use those lifecycle commands as the first inspection and recovery surface after
launch. The intended local path is `launch`, then `list` or `status` to inspect
what Port owns under `runtime/`, then `stop` when you want Port to end and
cleanly release that local runtime state.

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

Current artifact-command behavior:

- `port artifacts build` and `validate` operate on one selected artifact
  variant at a time.
- The variant vocabulary is explicit on the CLI:
  `--architecture`, `--substrate`, and `--protection-mode`.
- The sample config currently ships only Firecracker/standard variants, but the
  model and help text already reserve the same command surface for PVM, Cloud
  Hypervisor, and AVF lanes.
- `port artifacts push` writes the selected local variant into the configured
  backend and warms the local cache.
- `port artifacts pull` restores the selected variant from the configured
  backend into both the cache and the canonical local path used by `launch`.
- Firecracker/PVM is a separate future artifact kit on `x86_64`; do not assume
  the current `standard` kernel or guest image can be reused there.

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
- Artifact mobility is already designed around that future remote workflow:
  build or publish the selected variant on one Linux host, then pull the same
  logical reference onto another Linux host once the execution lane requires it.
- `port machine list`, `status`, and `stop` currently inspect only local
  Port-managed runtime roots; they do not yet enumerate or control
  remote/cloud hosts.
- The operator-visible ownership vocabulary is already aligned with hosted
  design work: local commands report `local-runtime-root` /
  `local-port-runtime` today, and future hosted drivers are expected to report
  `hosted-control-plane` / `hosted-node-agent` through the same fields.
- The future hosted split for lifecycle ownership and guest-operation brokering
  is documented in [`hosted.md`](hosted.md).
- The explicit Firecracker/PVM host-kit contract lives in [`pvm.md`](pvm.md).

The full cloud matrix and substrate lane guidance live in [`docs/cloud.md`](cloud.md).
The hosted node-agent/control-plane split that will eventually sit behind the
same CLI lives in [`docs/hosted.md`](hosted.md).

## macOS Workflow

The current shipped macOS workflow is still to run the actual Firecracker
commands on a Linux host, but Port now has an explicit first-class AVF
contract in [`avf.md`](avf.md).

Recommended path:

1. Keep the repository on the Linux host directly, or SSH into that host from macOS.
2. On the Linux host, run the same canonical commands shown in the Linux workflow.
3. Use `port doctor` on that Linux host before attempting local Firecracker launch.

Current boundary:

- Running local Firecracker launch directly on macOS is unsupported because the
  MVP launch path requires Linux and `/dev/kvm`.
- Apple Virtualization Framework is now a first-class planned Port lane, but it
  is not executable in the current runtime yet.
- The planned AVF lane keeps the same `machine` and `guest` verbs, maps guest
  transport onto AVF virtio sockets, and maps console/log capture onto AVF
  serial ports.
- Distributed macOS app targets will need Apple's virtualization entitlement;
  Rosetta-in-Linux-VM workflows are optional and depend on AVF directory
  sharing rather than replacing Port's guest-agent contract.
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

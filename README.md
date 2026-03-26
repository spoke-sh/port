# Port

Port is a CLI-first system for building, launching, and operating isolated
Linux workloads in microVMs across local and hosted environments.

It keeps one operator vocabulary across lanes:

- `port doctor` for host and lane checks
- `port artifacts` for build, validate, push, and pull
- `port machine` for lifecycle and status
- `port guest` for exec, copy, PTY, logs, and forward
- `port service` for secrets, services, and sandboxes

## Current Shape

- Default local lane: Firecracker with `standard` protection on Linux
- Hosted lane: control plane plus node agent with the same `machine`, `guest`,
  and `service` verbs
- SSH-managed remote lane: one bounded Linux lifecycle slice for `machine
  launch`, `status`, and `stop` through `mode = "ssh"` with explicit route and
  ownership output
- Attached volume first slice: one persistent `host-file` attached volume on
  the local Firecracker `standard` lane with explicit host path and ownership
  output
- Hosted stateless K3s first slice: one hosted control plane, one host group,
  one server machine, one or more worker machines, and cluster access through
  the canonical `machine` and `guest` verbs instead of a second Kubernetes-only
  command family
- Additional proof-backed lanes: Cloud Hypervisor `standard`, AVF `standard`,
  and prepared-node Firecracker/PVM on `x86_64`

## Quick Start

```bash
port doctor
port --config examples/port.toml artifacts build --artifact demo-kernel --architecture native
port --config examples/port.toml machine launch --machine demo
port --config examples/port.toml machine list
port --config examples/port.toml guest exec --machine demo -- /bin/sh -lc 'cat /proc/version'
```

Use `examples/port.toml` for the checked-in repo workflow. Detailed config
edits and longer examples now live in [`CONFIGURATION.md`](CONFIGURATION.md).
The first hosted stateless K3s workflow, boundaries, and proof command live in
[`docs/operators.md`](docs/operators.md).
The first direct-runtime attached-volume workflow and proof command live in
[`docs/operators.md`](docs/operators.md).
The first hosted external-project deployment proof path, repo-level review
surface, and current
boundaries also live in [`docs/operators.md`](docs/operators.md).
The first installable release contract and support matrix live in
[`docs/install.md`](docs/install.md).
Packaged macOS AVF workflows still use the canonical `port` CLI plus an
external `PORT_AVF_LAUNCHER` helper; distributed macOS targets remain bounded
by Apple's virtualization entitlement requirements described in
[`docs/avf.md`](docs/avf.md).

## Mission Report

```bash
just mission [<mission-id>]
```

That is the repo-level proof surface. It wraps the active mission in a
board-backed report with goal status, recent achievements, the current primary
demo path, and the recorded review artifact.

If you want the raw board entity output instead, run `keel mission show
<mission-id>` directly.

For the current hosted external-project deployment slice, `just mission` is the
repo-level review surface:

- it points at `bash scripts/hosted-external-project-demo.sh` as the runnable
  external-project workflow
- it points at `./scripts/render-external-project-proof.sh
  .keel/stories/VEyjdN0nf/EVIDENCE` plus the recorded GIF and cast artifact for
  review
- it assumes the repo dev shell so `port`, `port-guest-agent`, `busybox`,
  `curl`, and `agg` are available, with `PORT_DEMO_TOKEN` set or left at the
  repo default
- it proves one repo-local external static-site snapshot staged through hosted
  `port guest copy`, `port service apply`, and `port guest forward`
- it keeps app bundle artifact contracts and app bundle service runtimes as
  explicit follow-on work
- it stays named `mission` until upstream `keel screen` exists and Port can
  hard-cut to `keel screen`
- it uses the current renderer-backed cast/GIF path today; future `atxt`
  adoption remains explicit follow-on work

## Documentation Map

### Root Contracts

| Document | Purpose |
|----------|---------|
| [`CONSTITUTION.md`](CONSTITUTION.md) | Non-negotiable product and workflow rules |
| [`ARCHITECTURE.md`](ARCHITECTURE.md) | System boundaries, ownership, and major components |
| [`CONFIGURATION.md`](CONFIGURATION.md) | Config model, environment variables, and detailed workflow examples |
| [`RELEASE.md`](RELEASE.md) | Current release contract and validation checklist |
| [`EVALUATIONS.md`](EVALUATIONS.md) | Verification and evidence expectations |
| [`AGENTS.md`](AGENTS.md) | Shared AI-agent workflow contract |

### Focused Guides

| Document | Purpose |
|----------|---------|
| [`docs/operators.md`](docs/operators.md) | Operator-oriented overview and platform guidance |
| [`docs/install.md`](docs/install.md) | Installable release contract, support matrix, and package boundaries |
| [`docs/hosted.md`](docs/hosted.md) | Hosted control-plane, node-agent, and service workflows |
| [`docs/cloud.md`](docs/cloud.md) | Cloud-provider and hosted-lane boundaries |
| [`docs/artifacts.md`](docs/artifacts.md) | Artifact references, variants, and backends |
| [`docs/pvm.md`](docs/pvm.md) | Firecracker/PVM host-kit and artifact-kit contract |
| [`docs/avf.md`](docs/avf.md) | Apple Virtualization Framework lane |
| [`docs/sdk.md`](docs/sdk.md) | Hosted SDK and typed client surface |

## Platform Summary

| Environment | What Port supports today |
|-------------|--------------------------|
| Linux | Full local Firecracker workflow plus hosted control-plane demos |
| macOS | AVF local workflow through the canonical `machine` and `guest` verbs |
| Windows | Linux-backed workflow through WSL or a remote Linux host; no native install package in the first slice |

Use [`docs/install.md`](docs/install.md) for the installable release contract,
[`docs/operators.md`](docs/operators.md) for the platform guide, and
[`CONFIGURATION.md`](CONFIGURATION.md) for the detailed configuration and
workflow examples.

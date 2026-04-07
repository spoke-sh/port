# Port

Port is a CLI-first system for building, launching, and operating isolated
Linux workloads in microVMs across local and hosted environments.

It keeps one operator vocabulary across lanes:

- `port doctor` for host and lane checks
- `port cluster` for named cluster lifecycle and kubeconfig handoff
- `port artifacts` for build, validate, push, and pull
- `port machine` for lifecycle and status
- `port guest` for exec, copy, PTY, logs, and forward
- `port service` for secrets, services, and sandboxes

## Current Shape

- Default local lane: Firecracker with `standard` protection on Linux
- Hosted standard lane: control plane plus node agent with the same `machine`,
  `guest`, and `service` verbs, plus the first live hosted proofs
- Strongest current production-oriented cloud path: `cloud-aws` on a prepared
  `aws-linux-node` using `x86_64` Firecracker/PVM through the hosted control
  plane and node agent, with a read-only guest base image plus a writable
  overlay disk instead of a full rootfs copy on every launch retry
- SSH-managed remote lane: one bounded Linux lifecycle slice for `machine
  launch`, `status`, and `stop` through `mode = "ssh"` with explicit route and
  ownership output
- Local cluster first slice: one named local K3s cluster on the Firecracker
  `standard` lane with `port cluster up|status|kubeconfig|down`, Port-owned
  offline bootstrap inputs, and an explicit downstream handoff of health plus
  kubeconfig
- Hosted K3s cluster slice: named hosted K3s clusters under the same
  `port cluster` verbs, with Firecracker guest microVMs as the K3s nodes,
  explicit control-plane and worker machine sets, an explicit control-plane
  scheduler, and an operator-supplied HTTPS API endpoint
- Attached volume first slice: one persistent `host-file` attached volume on
  the local Firecracker `standard` lane with explicit host path and ownership
  output
- Additional proof-backed lanes: Cloud Hypervisor `standard`, AVF `standard`,
  and prepared-node Firecracker/PVM on `x86_64`

## Production Posture

If you need one cloud story to evaluate first, use the AWS hosted PVM path:

- canonical machine: `cloud-aws`
- canonical node: `aws-linux-node`
- canonical lane: `x86_64` + `firecracker` + `pvm`
- canonical readiness step: `port control-plane prepare-pvm-node`
- canonical lifecycle: `port machine launch`, `status`, and `stop`
- canonical guest storage mode: read-only `x86_64/firecracker/pvm` guest image
  plus a writable runtime overlay

What stays explicit:

- hosted `standard` Firecracker remains the easiest way to prove the control
  plane, node agent, guest, and service contract
- AWS PVM is the stronger production-oriented narrative because it carries the
  real prepared-host and no-fallback contract
- the Port flake now exports `nixosModules.aws-pvm-host` and
  `packages.x86_64-linux.firecracker-pvm-host-kit` as the supported downstream
  AWS PVM host-kit handoff
- Port still does not claim EC2 provisioning, IAM, VPC wiring, DNS, or arm64
  Firecracker/PVM support

Start with [`docs/aws.md`](docs/aws.md), then use
[`CONFIGURATION.md`](CONFIGURATION.md),
[`docs/hosted.md`](docs/hosted.md), and [`docs/pvm.md`](docs/pvm.md) for the
deeper contract.

## Installation

### One-liner Install (macOS and Linux)

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/spoke-sh/port/releases/latest/download/port-installer.sh | sh
port doctor
```

### Upgrade an Existing Install

```bash
port upgrade
port upgrade --tag <tag>
port upgrade --sha <git-sha>
```

### Manual Download

Download the latest pre-built binaries and installers for your platform from the
[GitHub Releases](https://github.com/spoke-sh/port/releases) page. We provide:
- **Linux:** `.tar.xz` archives plus the cross-platform shell installer
- **macOS:** `.tar.xz` archives plus the cross-platform shell installer
- **Windows:** not shipped in this slice; use WSL or a remote Linux host

## Quick Start

```bash
port doctor
port --config examples/port.toml cluster show --cluster demo
port --config examples/port.toml cluster up --cluster demo --runtime-root /tmp/port-runtime
port --config examples/port.toml cluster status --cluster demo --runtime-root /tmp/port-runtime
port --config examples/port.toml artifacts build --artifact demo-kernel --architecture native
port --config examples/port.toml machine list
```

Use `examples/port.toml` for the checked-in repo workflow. Detailed config
edits and longer examples now live in [`CONFIGURATION.md`](CONFIGURATION.md).
The clearest AWS production-oriented path now lives in [`docs/aws.md`](docs/aws.md).
The first local cluster workflow, thin downstream infra handoff, and proof
command live in
[`docs/operators.md`](docs/operators.md).
The first direct-runtime attached-volume workflow and proof command live in
[`docs/operators.md`](docs/operators.md).
The first hosted external-project deployment proof path, repo-level review
surface, and current
boundaries also live in [`docs/operators.md`](docs/operators.md).
The first hosted AWS PVM prepare-plus-launch proof path and review artifact
also live in [`docs/operators.md`](docs/operators.md).
The first installable release contract and support matrix live in
[`docs/install.md`](docs/install.md).
The public user-facing MDX docs site now lives in [`website/`](website/) and
can be run locally with `just docs-dev`.
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

For the current local cluster deployment-prep slice, `just mission` is the
repo-level review surface:

- it points at `./scripts/render-local-cluster-proof.sh
  .keel/stories/VFDk8ggoV/EVIDENCE` plus the recorded GIF and cast artifact for
  review
- it assumes the repo dev shell so `port`, `port-guest-agent`, `agg`, and the
  local Linux Firecracker lane are available
- it proves the canonical `port cluster up|status|kubeconfig|down` workflow for
  one named local K3s cluster
- it keeps the downstream seam thin: infra asks Port for cluster readiness plus
  kubeconfig, then owns later GitOps/bootstrap convergence
- it keeps hosted microVM-backed K3s as a separate contract, and it keeps real
  HA, richer networking, ingress, and storage guarantees as explicit follow-on
  work around that contract
- it stays named `mission` until upstream `keel screen` exists and Port can
  hard-cut to `keel screen`
- it uses the current renderer-backed cast/GIF path today; future `atxt`
  adoption remains explicit follow-on work

## Documentation Map

### Start Here

| If you need... | Start here |
|----------------|------------|
| the strongest current cloud narrative | [`docs/aws.md`](docs/aws.md) |
| the public narrative site | [`website/docs/path-to-production/aws.mdx`](website/docs/path-to-production/aws.mdx) |
| the downstream AWS AMI host-kit handoff | [`docs/aws.md`](docs/aws.md) |
| the local-first operator path | [`docs/operators.md`](docs/operators.md) |
| installation and support matrix | [`docs/install.md`](docs/install.md) |

### Root Contracts

| Document | Purpose |
|----------|---------|
| [`CONSTITUTION.md`](CONSTITUTION.md) | Non-negotiable product and workflow rules |
| [`ARCHITECTURE.md`](ARCHITECTURE.md) | System boundaries, ownership, and major components |
| [`CONFIGURATION.md`](CONFIGURATION.md) | Config model, environment variables, and detailed workflow examples |
| [`RELEASE.md`](RELEASE.md) | Current release contract and validation checklist |
| [`EVALUATIONS.md`](EVALUATIONS.md) | Verification and evidence expectations |
| [`AGENTS.md`](AGENTS.md) | Shared AI-agent workflow contract |
| [`website/`](website/) | Public Docusaurus site and user-facing MDX docs |

### Focused Guides

| Document | Purpose |
|----------|---------|
| [`docs/aws.md`](docs/aws.md) | Canonical AWS deployment, hosted PVM production contract, and downstream Nix host-kit handoff |
| [`docs/operators.md`](docs/operators.md) | Operator-oriented overview and platform guidance |
| [`docs/install.md`](docs/install.md) | Installable release contract, support matrix, and package boundaries |
| [`docs/hosted.md`](docs/hosted.md) | Hosted control-plane, node-agent, and service workflows |
| [`docs/cloud.md`](docs/cloud.md) | Cloud-provider matrix, standard hosted lane, and secondary cloud boundaries |
| [`docs/artifacts.md`](docs/artifacts.md) | Artifact references, variants, and backends |
| [`docs/pvm.md`](docs/pvm.md) | Firecracker/PVM host-kit and artifact-kit contract behind the AWS lane |
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

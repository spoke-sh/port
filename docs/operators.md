# Operator Guide

Use `port` for runtime workflows and `just mission` for a repo-level mission
report with recent achievements and human-facing artifacts. In the current
external-project deployment slice, `just mission` is the review surface for the
hosted static-site proof, while `port` remains the runtime surface that
actually launches, stages, exposes, and stops the workload.

## Platform Summary

| Environment | Supported path |
|-------------|----------------|
| Linux | Local Firecracker, hosted control-plane demos, and the first SSH-managed remote lifecycle slice |
| macOS | AVF local workflow through the same `machine` and `guest` verbs |
| Windows | Linux-backed workflow through WSL or a remote Linux host |

## Hybrid Execution Contract

Port keeps one operator vocabulary across three execution lanes:

| Host connection | Canonical route tokens | Operator workflow |
|-----------------|------------------------|-------------------|
| `mode = "local"` | `direct-local-runtime`, `local-runtime-root`, `local-port-runtime` | Run `port doctor`, then `port machine ...`, `port guest ...`, and `port service ...` directly on the local execution host |
| `mode = "hosted-control-plane"` | `hosted-control-plane`, `hosted-control-plane`, `hosted-node-agent` | Keep the same `port machine ...`, `guest ...`, and `service ...` verbs, but route them through the hosted control plane plus node agent |
| `mode = "ssh"` | `ssh-managed-remote`, `ssh-remote-runtime`, `ssh-remote-port-runtime` | Keep the same `port machine launch`, `status`, and `stop` verbs while Port shells into one remote Linux host that already exposes Port, Firecracker, and the selected artifact paths |

The command family stays canonical on purpose. Port does not introduce a
second remote-only CLI for the SSH lane.

## SSH-First Remote Linux Workflow

Use a host entry like this when one Linux machine should own the runtime
directly over SSH:

```toml
[hosts.generic-linux]
platform = "linux"
provider = "generic-linux"

[hosts.generic-linux.connection]
mode = "ssh"
destination = "builder.example.internal"
user = "ubuntu"
port = 2222

[hosts.generic-linux.firecracker]
local_launch = false
notes = ["Remote Linux host must already expose Port, Firecracker, and the selected artifact paths."]
```

Then keep the same lifecycle commands:

```bash
port --config /tmp/port-ssh.toml doctor
port --config /tmp/port-ssh.toml machine launch --machine cloud-generic --runtime-root /var/lib/port/runtime
port --config /tmp/port-ssh.toml machine status --machine cloud-generic --runtime-root /var/lib/port/runtime
port --config /tmp/port-ssh.toml machine stop --machine cloud-generic --runtime-root /var/lib/port/runtime
```

The SSH lane makes route ownership explicit in command output:

- `launch route: ssh-managed-remote`
- `inventory owner: ssh-remote-runtime`
- `lifecycle owner: ssh-remote-port-runtime`

This first SSH slice is intentionally narrow:

- `port doctor` explains SSH auth and bootstrap expectations before launch.
- `port machine launch`, `status`, and `stop` are implemented through the
  shared machine model.
- Guest operations, service operations, `machine monitor`, and `machine top`
  remain future SSH follow-on work.

## Attached Volume First Slice

Port now keeps one attached volume contract explicit instead of treating data
disks like alternate rootfs artifacts:

- one persistent `host-file` attached volume per machine
- one explicit host path owned by the launch route
- one visible ownership contract in `port doctor`, `machine launch`,
  `machine status`, and `machine stop`

The currently supported lane is intentionally narrow:

- local Firecracker with `standard` protection
- route: `direct-local-runtime`
- inventory owner: `local-runtime-root`
- lifecycle owner: `local-port-runtime`

Hosted-control-plane and SSH-managed machines reject attached volumes in this
slice with explicit lane guidance. Port will not silently reroute the request
or collapse the attached volume back into the guest image or rootfs contract.

Canonical direct-runtime workflow:

```bash
port --config /tmp/port-attached-volume.toml doctor
port --config /tmp/port-attached-volume.toml machine launch --machine demo
port --config /tmp/port-attached-volume.toml machine status --machine demo
port --config /tmp/port-attached-volume.toml machine stop --machine demo
```

The config for that workflow keeps the storage contract explicit:

```toml
[[machines.demo.volumes]]
name = "data"
backend = "host-file"
persistence = "persistent"
path = "/var/lib/port/volumes/demo-data.ext4"
```

Repo-local proof for this workflow:

```bash
./scripts/render-attached-volume-proof.sh .keel/stories/VDfF1dVOF/EVIDENCE
```

## Hosted Stateless K3s First Slice

Port's first K3s lane is intentionally narrow and stays on the hosted control
plane plus node-agent path. It does not introduce `port k3s` or a second
Kubernetes-only operator toolchain.

The contract is:

- one hosted control plane
- one host group
- one K3s server machine
- one or more worker machines
- stateless Firecracker `standard` machines only

Config shape:

```toml
[k3s_clusters.demo]
control_plane = "demo"
host_group = "remote-linux"
server_machine = "cloud-generic"
worker_machines = ["cloud-aws"]
version = "v1.32.0+k3s1"
server_args = ["--disable=traefik"]
worker_args = ["--node-label=role=worker"]
```

Canonical hosted workflow:

```bash
export PORT_DEMO_TOKEN=demo-token
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-k3s.toml control-plane serve --control-plane demo --bind 127.0.0.1:7040
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-k3s.toml node-agent serve --node generic-linux-node --bind 127.0.0.1:9234 --token node-secret
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-k3s.toml node-agent serve --node aws-linux-node --bind 127.0.0.1:9235 --token node-secret
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-k3s.toml machine launch --machine cloud-generic
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-k3s.toml machine launch --machine cloud-aws
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-k3s.toml guest exec --machine cloud-generic -- /bin/sh -lc "curl -sfL https://get.k3s.io | INSTALL_K3S_VERSION='v1.32.0+k3s1' INSTALL_K3S_EXEC='server --disable=traefik' sh -"
JOIN_TOKEN="$(PORT_DEMO_TOKEN=demo-token port --config /tmp/port-k3s.toml guest exec --machine cloud-generic -- /bin/sh -lc 'cat /var/lib/rancher/k3s/server/node-token')"
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-k3s.toml guest exec --machine cloud-aws -- /bin/sh -lc "curl -sfL https://get.k3s.io | INSTALL_K3S_VERSION='v1.32.0+k3s1' K3S_URL='https://cloud-generic:6443' K3S_TOKEN='${JOIN_TOKEN}' INSTALL_K3S_EXEC='agent --node-label=role=worker' sh -"
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-k3s.toml guest exec --machine cloud-generic -- /bin/sh -lc 'cat /etc/rancher/k3s/k3s.yaml'
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-k3s.toml guest exec --machine cloud-generic -- /bin/sh -lc 'k3s kubectl get nodes -o wide'
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-k3s.toml machine stop --machine cloud-aws
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-k3s.toml machine stop --machine cloud-generic
```

First-slice boundaries stay explicit:

- no HA or multi-server control planes
- no attached volumes, persistent storage, or CSI
- no ingress, load balancers, or public service exposure
- no SSH-owned or multi-group cluster routing

Repo-local proof for this workflow:

```bash
./scripts/render-hosted-k3s-proof.sh .keel/stories/VDfzOEeFL/EVIDENCE
```

## Hosted External Project Deployment First Slice

Port now has one bounded answer to "can it host an external project?" without
claiming a general hosted platform or a container-like app bundle surface.

Review surface:

```bash
just mission
```

Runnable hosted workflow:

```bash
bash scripts/hosted-external-project-demo.sh
```

That proof path keeps the operator contract explicit:

- repo-local hosted control plane plus node agent
- one hosted machine: `cloud-aws`
- one explicit host group: `aws-builders`
- one repo-local external static-site snapshot:
  `examples/external-static-site/index.html`
- one staging path through hosted `port guest copy`
- one minimal HTTP service launched through `port service apply`
- one host-side exposure through `port guest forward`
- one host-side `curl` proving the payload

Human-reviewable artifact:

```bash
./scripts/render-external-project-proof.sh .keel/stories/VEyjdN0nf/EVIDENCE
```

The current proof prerequisites are intentionally narrow:

- run from the repo dev shell so `port`, `port-guest-agent`, `busybox`, `curl`,
  and `agg` are available
- keep `PORT_DEMO_TOKEN` available for the repo-local hosted control-plane
  contract, or rely on the script's repo-default `demo-token`
- treat the shipped workflow as a repo-local proof lane, not external hosted
  infrastructure

First-slice boundaries stay explicit:

- current repo-level entrypoint name is `mission`; future `screen` cutover is
  separate work once upstream `keel screen` ships
- current recording path is the checked-in renderer plus cast/GIF artifact;
  future `atxt` migration is separate work
- this slice stages and runs one external static-site project snapshot through
  shipped hosted primitives only; it does not yet ship an app bundle artifact
  contract or app bundle service runtime
- this slice does not ship ingress, public exposure, multi-service
  orchestration, autoscaling, tenancy, or production-hosting guarantees

## SSH Repo-local Proof

The checked-in proof command for this workflow is:

```bash
./scripts/render-hybrid-ssh-proof.sh .keel/stories/VDeuzbve3/EVIDENCE
```

That script generates a deterministic asciicast plus a terminal-renderable GIF
for mission review. It uses a simulated SSH transport and fake Firecracker
binary so the proof stays stable while the operator-facing CLI contract remains
the real `port doctor` plus `port machine launch|status|stop` workflow.

## Common Examples

```bash
port doctor
port --config examples/port.toml machine launch --machine demo
port --config examples/port.toml machine list
port --config examples/port.toml guest exec --machine demo -- /bin/sh -lc 'cat /proc/version'
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml machine status --machine cloud-aws
```

## Where To Go Next

- Detailed config edits and longer examples:
  [`CONFIGURATION.md`](../CONFIGURATION.md)
- Hosted control-plane and service workflows:
  [`hosted.md`](hosted.md)
- Cloud lanes and provider boundaries:
  [`cloud.md`](cloud.md)
- Artifact references and backend contracts:
  [`artifacts.md`](artifacts.md)
- Firecracker/PVM:
  [`pvm.md`](pvm.md)
- Apple Virtualization Framework:
  [`avf.md`](avf.md)

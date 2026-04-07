# Operator Guide

Use `port` for runtime workflows and `just mission` for a repo-level mission
report with recent achievements and human-facing artifacts. In the current
local-cluster deployment-prep slice, `just mission` is the review surface for
the human-reviewable cluster proof, while `port` remains the runtime surface
that actually brings the cluster up, reports readiness, hands off kubeconfig,
and tears it down.

## Choose Your Starting Point

- If you are evaluating a production-shaped AWS rollout, start with
  [`aws.md`](aws.md). That is the clearest current cloud path and the canonical
  hosted `x86_64` Firecracker/PVM narrative.
- If you are proving the product locally first, start with the local cluster
  slice in this guide.
- If you are learning the hosted control-plane split before the prepared AWS
  path, use [`hosted.md`](hosted.md).

## Platform Summary

| Environment | Supported path |
|-------------|----------------|
| Linux | Local Firecracker, the first local cluster lifecycle slice, hosted control-plane demos, the first SSH-managed remote lifecycle slice, and the strongest current AWS `x86_64` hosted PVM path |
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

## Local Cluster First Slice

Port's first blessed cluster workflow is now local, single-node, and
cluster-first. Operators should use `port cluster ...` for the first K3s lane
instead of assembling cluster bring-up from `machine launch`, `guest exec`,
manual API forwarding, or kubeconfig rewriting.

The contract is:

- one named cluster from `[clusters.<name>]`
- provider `local`
- count `1`
- one Firecracker `standard` machine on the Linux local lane
- Port-owned offline bootstrap inputs
- Port-owned readiness reporting and kubeconfig handoff

Config shape:

```toml
[clusters.demo]
flavor = "k3s"
provider = "local"
count = 1
machine = "demo"
version = "v1.32.2+k3s1"
args = ["--disable=traefik"]

[clusters.demo.bootstrap]
stage_root = "/opt/port/clusters/demo"
install_script = "examples/bootstrap/demo-k3s/install-k3s-offline.sh"
binary = "examples/bootstrap/demo-k3s/k3s"

[clusters.demo.bootstrap.guest_profile]
name = "kube-ready"
required_commands = ["sh", "install", "ln", "chmod"]

[clusters.demo.lifecycle]
health_command = ["opt/port/clusters/demo/bin/k3s", "kubectl", "get", "nodes", "-o", "wide"]
kubeconfig_path = "/etc/rancher/k3s/k3s.yaml"
api_forward_target = "127.0.0.1:6443"
```

Canonical workflow:

```bash
port --config /tmp/port-local-cluster.toml cluster show --cluster demo
port --config /tmp/port-local-cluster.toml cluster up --cluster demo --runtime-root /var/lib/port/runtime
port --config /tmp/port-local-cluster.toml cluster status --cluster demo --runtime-root /var/lib/port/runtime
port --config /tmp/port-local-cluster.toml cluster kubeconfig --cluster demo --runtime-root /var/lib/port/runtime --format json
port --config /tmp/port-local-cluster.toml cluster down --cluster demo --runtime-root /var/lib/port/runtime
```

Thin downstream infra handoff:

- Port owns machine launch, offline K3s staging, guest bootstrap, readiness,
  API forward lifecycle, and kubeconfig server rewrite for this first slice.
- Downstream infra asks Port for `cluster status` and `cluster kubeconfig`, then
  owns everything after that point: Flux bootstrap, Pulumi operator setup,
  GitOps convergence, and broader platform health.
- Raw `machine launch`, `guest exec`, `guest forward`, and manual kubeconfig
  edits remain implementation substrate or troubleshooting tools, not the
  blessed cluster workflow.

First-slice boundaries stay explicit:

- single-node local only
- no hosted, multi-node, or AWS cluster orchestration
- no guest networking, CIDR management, or stable inter-node addressing
- no ingress, load balancers, public service exposure, attached volumes, or
  persistent storage guarantees
- `port cluster stage` remains a diagnostic or proof substrate, not the primary
  operator handoff

Human-reviewable artifact:

```bash
./scripts/render-local-cluster-proof.sh .keel/stories/VFDk8ggoV/EVIDENCE
```

## Hosted K3s MicroVM Contract

Hosted K3s now uses the same `port cluster ...` verbs, but the nodes are
hosted guest microVMs instead of one local guest.

Config shape:

```toml
[k3s_clusters.demo]
control_plane = "demo"
host_group = "aws-builders"
server_machines = ["cloud-aws-a", "cloud-aws-b", "cloud-aws-c"]
worker_machines = ["cloud-aws-worker-a", "cloud-aws-worker-b"]
api_endpoint = "https://demo-k3s.internal:6443"
version = "v1.32.0+k3s1"
server_args = ["--disable=traefik"]
worker_args = ["--node-label=role=worker"]
```

Canonical workflow:

```bash
port --config /tmp/port-hosted-k3s.toml cluster show --cluster demo
port --config /tmp/port-hosted-k3s.toml cluster up --cluster demo --runtime-root /var/lib/port/runtime
port --config /tmp/port-hosted-k3s.toml cluster status --cluster demo --runtime-root /var/lib/port/runtime
port --config /tmp/port-hosted-k3s.toml cluster kubeconfig --cluster demo --runtime-root /var/lib/port/runtime
port --config /tmp/port-hosted-k3s.toml cluster down --cluster demo --runtime-root /var/lib/port/runtime
```

Interpret that contract this way:

- the execution hosts, including AWS PVM hosts, run Port node-agent ownership
- the K3s control-plane and worker nodes are the guest microVMs named in
  `server_machines` and `worker_machines`
- `api_endpoint` must already front the control-plane microVMs through an
  external load balancer or VIP

Real-HA boundary:

- at least three control-plane microVMs
- those control-plane microVMs spread across distinct execution hosts
- one stable HTTPS API endpoint fronting them

Port bootstraps and reports that topology, but it does not ship the external
load balancer, VIP, DNS, ingress, or storage layer around it.

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

- run from the repo dev shell so `port`, `port-guest-agent`, `curl`, and `agg`
  are available and `PORT_BUSYBOX_BIN` is set, or install `busybox` on the host
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

## Hosted AWS PVM Repo-local Proof

Runnable hosted workflow:

```bash
bash scripts/hosted-pvm-demo.sh
```

Human-reviewable artifact:

```bash
./scripts/render-hosted-pvm-proof.sh .keel/stories/VFgcoUoUd/EVIDENCE
```

That proof path keeps the operator contract explicit:

- one hosted machine: `cloud-aws`
- one hosted node: `aws-linux-node`
- one canonical readiness step: `port control-plane prepare-pvm-node`
- one imported ready record under `.port/hosted/demo/imported-inventory.json`
- one repo-local fake `firecracker-pvm` plus temporary `x86_64/firecracker/pvm`
  kernel and guest artifact paths so the proof stays reproducible

Current boundaries stay explicit:

- `x86_64` AWS hosted PVM only
- no generic fallback and no inheritance to GCP or Azure
- no arm64 Firecracker/PVM claim
- this remains a repo-local hosted control-plane plus node-agent proof, not
  external AWS infrastructure provisioning

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
port --config examples/port.toml cluster show --cluster demo
port --config examples/port.toml cluster status --cluster demo --runtime-root /tmp/port-runtime
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

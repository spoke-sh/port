# Operator Guide

Use `port` for runtime workflows and `just mission` for a repo-level mission
report with recent achievements and human-facing artifacts.

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

## Repo-local Proof

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

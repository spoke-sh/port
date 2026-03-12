# Cloud Linux Support

Port's cloud Linux story is now split across provider identity and execution
lane. The current shipped Linux execution surface spans four proof-backed
paths: Firecracker with `standard` protection, the first SSH-managed remote
Linux lifecycle slice, prepared-node Firecracker/PVM on `x86_64`, and Cloud
Hypervisor with `standard` protection. The shared model still represents
follow-on or research-backed lanes beyond that shipped set.

The hosted control-plane split that will eventually carry these remote lanes is
defined in [`docs/hosted.md`](hosted.md).
The dedicated Firecracker/PVM host-kit contract lives in [`pvm.md`](pvm.md).
The dedicated AVF macOS contract lives in [`avf.md`](avf.md).

`port doctor` reports both provider-aware and lane-aware support boundaries, and
`port machine launch` fails fast only when you target a lane that Port still
models but does not execute yet.

## SSH-Managed Remote Linux Workflow

The first direct remote Linux lane now keeps the same machine command family
while switching the host connection to SSH:

```toml
[hosts.generic-linux.connection]
mode = "ssh"
destination = "builder.example.internal"
user = "ubuntu"
port = 2222
```

Once the remote host already has Port, Firecracker, and the selected artifact
paths installed, the operator workflow stays canonical:

```bash
port --config /tmp/port-ssh.toml doctor
port --config /tmp/port-ssh.toml machine launch --machine cloud-generic --runtime-root /var/lib/port/runtime
port --config /tmp/port-ssh.toml machine status --machine cloud-generic --runtime-root /var/lib/port/runtime
port --config /tmp/port-ssh.toml machine stop --machine cloud-generic --runtime-root /var/lib/port/runtime
```

Port makes the SSH lane explicit in output with route and ownership fields such
as `ssh-managed-remote`, `ssh-remote-runtime`, and
`ssh-remote-port-runtime`. The first SSH slice does not add a second command
family and does not yet cover guest or service workflows.

## Execution Lane Matrix

| Lane | Architectures | Current status | Notes |
|------|---------------|----------------|-------|
| Firecracker + `standard` | `x86_64`, `aarch64`, `native` | Supported today | The only shipped execution lane behind the current Linux launch workflow |
| Firecracker + `pvm` | `x86_64` | Hosted / partial | Strategic lane for cloud cost control; requires a prepared host kit, dedicated artifact variants, and the hosted control-plane plus node-agent path |
| Firecracker + `pvm` | `aarch64` | Research lane | Upstream protected virtualization exists, but Port does not yet claim a supportable Firecracker runtime path |
| Cloud Hypervisor + `standard` | `x86_64`, `aarch64` | Local + hosted / partial | Lifecycle and guest flows now reuse the canonical machine and guest verbs through the local driver boundary and the hosted control-plane plus node-agent path |
| AVF + `standard` | macOS `arm64` or `x86_64` | Local / partial | First-class macOS lane; keeps the canonical guest protocol over AVF virtio sockets and console capture over AVF serial ports |

## Provider Matrix

| Provider token | Example host | Example machine | MVP status | Current CLI behavior |
|----------------|--------------|-----------------|------------|----------------------|
| `local` | `local` | `demo` | Supported | `port doctor` runs full local preflight and `port machine launch --machine demo` can boot Firecracker on Linux |
| `generic-linux` | `generic-linux` | `cloud-generic` | Hosted standard lane / partial implementation | `port doctor` reports provider and lane detail; with `port control-plane serve`, a registered `generic-linux-node`, and standard artifacts, `port machine launch --machine cloud-generic` routes through the hosted control plane and selected node |
| `aws` | `aws-linux` | `cloud-aws` | Hosted standard lane / partial implementation | `port doctor` reports AWS readiness detail; with `port control-plane serve`, a registered `aws-linux-node`, and standard artifacts, `port machine launch --machine cloud-aws` routes through the hosted control plane and selected node |
| `gcp` | `gcp-linux` | `cloud-gcp` | Hosted standard lane / partial implementation | `port doctor` reports GCP readiness detail; with `port control-plane serve`, a registered `gcp-linux-node`, and standard artifacts, `port machine launch --machine cloud-gcp` routes through the hosted control plane and selected node |
| `azure` | `azure-linux` | `cloud-azure` | Unsupported for MVP | `port doctor` reports Azure as unsupported for Firecracker MVP and `port machine launch` rejects it immediately |

## Remote Linux Workflow

Use the hosted control-plane lane to run the shipped standard cloud workflow.

1. Keep the canonical config explicit about provider identity, for example `provider = "aws"` on `hosts.aws-linux`. Remote/cloud hosts point at the named hosted control plane with `mode = "hosted-control-plane"` plus `control_plane = "demo"`.
2. Run `port doctor --config examples/port.toml` on the Linux environment that will host the node agent and Firecracker execution.
3. Start the demo control plane and one node agent that owns a real runtime root:

   ```bash
   export PORT_DEMO_TOKEN=demo-token
   PORT_DEMO_TOKEN=demo-token port --config examples/port.toml control-plane serve --control-plane demo --bind 127.0.0.1:7040
   PORT_DEMO_TOKEN=demo-token port --config examples/port.toml node-agent serve --node aws-linux-node --bind 127.0.0.1:9234 --token node-secret
   ```

4. Launch, inspect, and stop the standard cloud machine through the canonical CLI:

   ```bash
   PORT_DEMO_TOKEN=demo-token port --config examples/port.toml machine launch --machine cloud-aws
   PORT_DEMO_TOKEN=demo-token port --config examples/port.toml machine status --machine cloud-aws
   PORT_DEMO_TOKEN=demo-token port --config examples/port.toml machine stop --machine cloud-aws
   ```

5. Use the same hosted flow for `cloud-generic` with `generic-linux-node` or for `cloud-gcp` with `gcp-linux-node`.
6. Use the repo-local proof to verify the shipped standard lane without hand-running the full server harness:

   ```bash
   cargo test -q -p port-cli --test machine_commands cli_hosted_standard_cloud_launch_round_trip
   cargo test -q -p port-cli --test machine_commands cli_hosted_standard_status_and_stop_round_trip
   ```

7. Prepared-node PVM remains a second hosted lane: switch `cloud-aws` to `protection_mode = "pvm"` only when the prepared host kit and PVM artifact paths from [`pvm.md`](pvm.md) exist.

## Local Cloud Hypervisor Workflow

Use the checked-in `machines.demo-ch` lane when you want a Linux-local Cloud
Hypervisor proof through the canonical CLI:

```bash
port --config examples/port.toml doctor
port --config examples/port.toml machine launch --machine demo-ch
port --config examples/port.toml machine status --machine demo-ch
port --config examples/port.toml guest exec --machine demo-ch -- /bin/sh -lc 'uname -a'
port --config examples/port.toml guest copy --machine demo-ch --direction host-to-guest --source ./host.txt --destination /workspace/host.txt
port --config examples/port.toml guest pty --machine demo-ch -- /bin/sh -lc 'printf ch-pty-ok'
port --config examples/port.toml guest logs --machine demo-ch --path /var/log/app.log --tail-lines 20
port --config examples/port.toml guest forward --machine demo-ch --listen 127.0.0.1:8080 --target 127.0.0.1:80
port --config examples/port.toml machine stop --machine demo-ch
```

Repository-local proof for the shipped local lane:

```bash
cargo test -q -p port-runtime cloud_hypervisor_launch_status_and_stop_write_canonical_runtime_state
cargo test -q -p port-runtime cloud_hypervisor_launch_surfaces_missing_binary_preflight
cargo test -q -p port-runtime guest_exec_uses_cloud_hypervisor_vsock_tunnel_when_runtime_socket_is_absent
cargo test -q -p port-cli --test machine_commands cli_machine_launch_status_and_stop_route_cloud_hypervisor_locally
cargo test -q -p port-cli --test machine_commands cli_machine_launch_surfaces_missing_cloud_hypervisor_binary
cargo test -q -p port-cli --test guest_commands cli_cloud_hypervisor_guest_commands_cover_all_capabilities
```

## Hosted Cloud Hypervisor Workflow

Keep `examples/port.toml` on the standard Firecracker hosted lane. For the
Cloud Hypervisor proof, copy the sample to a temporary config and make these
explicit changes:

1. Point `[control_planes.demo].endpoint` at `http://127.0.0.1:7040`.
2. Set `machines.cloud-aws.substrate = "cloud-hypervisor"`.
3. Set `machines.cloud-aws.architecture = "x86_64"`.
4. Set `nodes.aws-linux-node.capabilities.substrates = ["cloud-hypervisor"]`.
5. Keep `nodes.aws-linux-node.capabilities.protection_modes = ["standard"]`.

Then use the same hosted control-plane and node-agent verbs:

```bash
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-cloud-hypervisor.toml control-plane serve --control-plane demo --bind 127.0.0.1:7040
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-cloud-hypervisor.toml node-agent serve --node aws-linux-node --bind 127.0.0.1:9234 --token node-secret
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-cloud-hypervisor.toml machine launch --machine cloud-aws
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-cloud-hypervisor.toml machine status --machine cloud-aws
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-cloud-hypervisor.toml guest exec --machine cloud-aws -- /bin/sh -lc 'uname -a'
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-cloud-hypervisor.toml machine stop --machine cloud-aws
```

Repository-local proof for the hosted Cloud Hypervisor lane:

```bash
cargo test -q -p port-runtime hosted_cloud_hypervisor_launch_status_stop_route_through_live_control_plane
cargo test -q -p port-runtime hosted_cloud_hypervisor_launch_rejects_firecracker_only_nodes_without_fallback
cargo test -q -p port-runtime hosted_guest_exec_routes_cloud_hypervisor_machine_through_node_runtime_root
cargo test -q -p port-cli --test machine_commands cli_hosted_cloud_hypervisor_launch_status_and_stop_round_trip
cargo test -q -p port-cli --test machine_commands cli_hosted_cloud_hypervisor_launch_rejects_firecracker_only_nodes_without_fallback
cargo test -q -p port-cli --test guest_commands cli_guest_commands_cover_hosted_cloud_hypervisor_runtime
```

Unsupported boundaries stay explicit:

- leaving `aws-linux-node` on `substrates = ["firecracker"]` is the explicit
  rejected-node proof for hosted Cloud Hypervisor; Port does not fall back
- Cloud Hypervisor is a `standard`-only Port lane today
- Azure remains unsupported for the current hosted Linux MVP
- `aarch64` Cloud Hypervisor remains modeled in artifacts and machine contracts,
  but the published hosted proof is the `x86_64` sample lane

## Operator Mapping

- Linux operators can use `port doctor` to inspect both local prerequisites and remote-provider intent from the same config.
- macOS operators should treat a Linux host as the execution environment for the Firecracker hosted lane and use the same canonical `port doctor`, `control-plane serve`, `node-agent serve`, and `port machine launch` commands there.
- Windows operators should use WSL or a remote Linux host for that same hosted workflow, then rely on `port doctor` to distinguish a usable Linux execution environment from an unsupported provider lane.

## Hosted Mapping

The planned hosted product uses the same command model, but with different
runtime ownership:

- `port machine list`, `status`, and `stop` are the local lifecycle surfaces
  that a future hosted control plane will remote behind one canonical CLI
  vocabulary.
- Those lifecycle commands now publish the local control-contract fields
  directly: `local-runtime-root`, `local-port-runtime`,
  `runtime-manifest-and-host-process`, and `direct-local-runtime`.
- node agents will own host-local hypervisor processes, runtime roots, and
  guest-transport attachment on execution hosts.
- the central control plane will own inventory, desired state, placement, and
  policy instead of the CLI process owning those concerns directly.
- the first hosted auth slice is already modeled explicitly through
  `[control_planes.<name>]`, including endpoint, audience, auth header, and
  token source.

The detailed hosted control contract lives in [`docs/hosted.md`](hosted.md).

## PVM Lane

The PVM / protected VM / confidential VM lane is back in scope, but under an
explicitly narrower contract than Port's overall architecture story:

- Firecracker/PVM on `x86_64` is the near-term implementation lane.
- That x86_64 lane depends on a prepared host kit: custom host kernel,
  patched Firecracker build, `pti=off`, and dedicated PVM artifact variants.
- Firecracker/PVM on `aarch64` remains a research lane until Port has a
  supportable runtime path rather than only upstream protected-virtualization
  evidence.
- Port will fail fast when machine or artifact compatibility claims a substrate,
  protection-mode, or architecture combination that the current lane does not
  support yet.

See [`pvm.md`](pvm.md) for the explicit host-kit, artifact-kit, validation, and
follow-on implementation contract.

That is the intended product posture: no hidden promises, no silent fallback,
and no conflation of "arm64 protected virtualization exists upstream" with
"Port ships an arm64 Firecracker/PVM runtime today."

# Cloud Linux Support

Port's cloud Linux story is now split across provider identity and execution
lane. The current executable lane remains Firecracker with `standard`
protection on Linux hosts; the shared model also represents planned or
research-backed substrate lanes such as Firecracker/PVM, Cloud Hypervisor, and
Apple Virtualization Framework.

The hosted control-plane split that will eventually carry these remote lanes is
defined in [`docs/hosted.md`](hosted.md).
The dedicated Firecracker/PVM host-kit contract lives in [`pvm.md`](pvm.md).
The dedicated AVF macOS contract lives in [`avf.md`](avf.md).

`port doctor` reports both provider-aware and lane-aware support boundaries, and
`port machine launch` fails fast only when you target a lane that Port still
models but does not execute yet.

## Execution Lane Matrix

| Lane | Architectures | Current status | Notes |
|------|---------------|----------------|-------|
| Firecracker + `standard` | `x86_64`, `aarch64`, `native` | Supported today | The only shipped execution lane behind the current Linux launch workflow |
| Firecracker + `pvm` | `x86_64` | Planned / partial design | Strategic lane for cloud cost control; requires dedicated host-kernel, VMM, and artifact work |
| Firecracker + `pvm` | `aarch64` | Research lane | Upstream protected virtualization exists, but Port does not yet claim a supportable Firecracker runtime path |
| Cloud Hypervisor + `standard` | `x86_64`, `aarch64` | Planned | Secondary Linux hypervisor lane, not yet implemented |
| AVF + `standard` | macOS `arm64` or `x86_64` | Planned | First-class macOS lane; keeps the canonical guest protocol over AVF virtio sockets and console capture over AVF serial ports |

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

# Cloud Linux Support

Port's cloud Linux story is now split across provider identity and execution
lane. The current executable lane remains Firecracker with `standard`
protection on Linux hosts; the shared model also represents planned or
research-backed substrate lanes such as Firecracker/PVM, Cloud Hypervisor, and
Apple Virtualization Framework.

The hosted control-plane split that will eventually carry these remote lanes is
defined in [`docs/hosted.md`](hosted.md).

`port doctor` reports both provider-aware and lane-aware support boundaries, and
`port machine launch` still fails fast when you target a lane that Port does not
yet execute.

## Execution Lane Matrix

| Lane | Architectures | Current status | Notes |
|------|---------------|----------------|-------|
| Firecracker + `standard` | `x86_64`, `aarch64`, `native` | Supported today | The only shipped execution lane behind the current Linux launch workflow |
| Firecracker + `pvm` | `x86_64` | Planned / partial design | Strategic lane for cloud cost control; requires dedicated host-kernel, VMM, and artifact work |
| Firecracker + `pvm` | `aarch64` | Research lane | Upstream protected virtualization exists, but Port does not yet claim a supportable Firecracker runtime path |
| Cloud Hypervisor + `standard` | `x86_64`, `aarch64` | Planned | Secondary Linux hypervisor lane, not yet implemented |
| AVF + `standard` | macOS `arm64` or `x86_64` | Planned | First-class macOS lane in the model and docs, not yet implemented |

## Provider Matrix

| Provider token | Example host | Example machine | MVP status | Current CLI behavior |
|----------------|--------------|-----------------|------------|----------------------|
| `local` | `local` | `demo` | Supported | `port doctor` runs full local preflight and `port machine launch --machine demo` can boot Firecracker on Linux |
| `generic-linux` | `generic-linux` | `cloud-generic` | Designed, partial implementation | `port doctor` reports the future remote Linux lane; `port machine launch` tells you to run Port on that Linux host directly |
| `aws` | `aws-linux` | `cloud-aws` | Designed, partial implementation | `port doctor` reports AWS as a justified future lane; `port machine launch --machine cloud-aws` returns AWS-specific not-yet-implemented guidance |
| `gcp` | `gcp-linux` | `cloud-gcp` | Designed, partial implementation | `port doctor` reports GCP as a justified future lane; `port machine launch` returns GCP-specific not-yet-implemented guidance |
| `azure` | `azure-linux` | `cloud-azure` | Unsupported for MVP | `port doctor` reports Azure as unsupported for Firecracker MVP and `port machine launch` rejects it immediately |

## Remote Linux Workflow

Use the cloud lane to model intent and inspect support boundaries, not to
perform remote launch yet.

1. Keep the canonical config explicit about provider identity, for example `provider = "aws"` on `hosts.aws-linux`.
2. Run `port doctor --config examples/port.toml` on the Linux environment you plan to use for Firecracker execution.
3. Read the provider-aware checks to confirm whether the target lane is `local`, a future remote lane (`generic-linux`, `aws`, `gcp`), or explicitly unsupported (`azure`).
4. For the current MVP, run `port machine launch --machine demo` only on a Linux host that passes `port doctor`.
5. If you try `port machine launch --machine cloud-aws` or another remote machine, Port intentionally fails fast with guidance about the current boundary.
6. `port machine list`, `port machine status`, and `port machine stop` currently inspect local runtime roots only; they are not yet a remote-cloud inventory surface.

## Operator Mapping

- Linux operators can use `port doctor` to inspect both local prerequisites and remote-provider intent from the same config.
- macOS operators should treat a Linux host as the execution environment and use the same canonical `port doctor` and `port machine launch` commands there.
- Windows operators should use WSL or a remote Linux host for the same workflow, then rely on `port doctor` to distinguish a usable Linux launch environment from a documentation-only remote lane.

## Hosted Mapping

The planned hosted product uses the same command model, but with different
runtime ownership:

- `port machine list`, `status`, and `stop` are the local lifecycle surfaces
  that a future hosted control plane will remote behind one canonical CLI
  vocabulary.
- node agents will own host-local hypervisor processes, runtime roots, and
  guest-transport attachment on execution hosts.
- the central control plane will own inventory, desired state, placement, and
  policy instead of the CLI process owning those concerns directly.

The detailed hosted control contract lives in [`docs/hosted.md`](hosted.md).

## PVM Lane

The PVM / protected VM / confidential VM lane is back in scope, but under an
explicitly narrower contract than Port's overall architecture story:

- Firecracker/PVM on `x86_64` is the near-term implementation lane.
- Firecracker/PVM on `aarch64` remains a research lane until Port has a
  supportable runtime path rather than only upstream protected-virtualization
  evidence.
- Port will fail fast when machine or artifact compatibility claims a substrate,
  protection-mode, or architecture combination that the current lane does not
  support yet.

That is the intended product posture: no hidden promises, no silent fallback,
and no conflation of "arm64 protected virtualization exists upstream" with
"Port ships an arm64 Firecracker/PVM runtime today."

# Cloud Linux Support

Port's cloud Linux lane is intentionally partial in the MVP. The shared model
and canonical CLI now represent remote Linux providers explicitly, `port doctor`
reports their current support boundary, and `port machine launch` fails fast
with provider-aware guidance when you target a remote cloud host.

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

## Operator Mapping

- Linux operators can use `port doctor` to inspect both local prerequisites and remote-provider intent from the same config.
- macOS operators should treat a Linux host as the execution environment and use the same canonical `port doctor` and `port machine launch` commands there.
- Windows operators should use WSL or a remote Linux host for the same workflow, then rely on `port doctor` to distinguish a usable Linux launch environment from a documentation-only remote lane.

## PVM Decision

The PVM / protected VM / confidential VM lane is dropped from the MVP.

Current research does not justify keeping it in scope: Firecracker still needs a
supportable Linux KVM path, while protected/confidential VM offerings change the
virtualization boundary in ways that are not presently compatible with the Port
MVP launch lane. If future research changes that conclusion, it should reopen as
new planning work rather than as hidden MVP scope.

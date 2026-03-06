---
created_at: 2026-03-06T15:37:00
---

# Reflection - Build Artifact Pipelines And Docs

## Knowledge

### 1vyeN0000: Build Firecracker Rootfs Images Without Privileged Mounts
| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When Port needs to build a minimal guest image inside the repository or CI without root-only mount steps |
| **Insight** | `mkfs.ext4 -d` plus `ldd`-discovered shared libraries is enough to assemble a bootable ext4 guest image carrying dynamic binaries like BusyBox and `port-guest-agent` |
| **Suggested Action** | Prefer staging-directory image assembly with `mkfs.ext4 -d`, `e2fsck`, and `debugfs` before introducing mount-based image mutation tooling |
| **Applies To** | `scripts/artifacts/*.sh`, guest image pipelines, future cloud image assembly |
| **Linked Knowledge IDs** | |
| **Observed At** | 2026-03-06T23:37:00Z |
| **Score** | 0.86 |
| **Confidence** | 0.93 |
| **Applied** | yes |

## Observations

- The cleanest kernel pipeline was a pinned Firecracker CI kernel fetch with architecture-specific digests rather than trying to compile a kernel from source inside the MVP slice.
- Building the guest image in-repo was practical without privileged mounts once the pipeline staged BusyBox, `port-guest-agent`, and their runtime libraries into a directory and let `mkfs.ext4 -d` materialize the filesystem.
- Running `port machine launch` against the built artifacts was worth the extra verification step because it proved the artifact outputs were usable for the real launch path rather than just structurally valid on disk.

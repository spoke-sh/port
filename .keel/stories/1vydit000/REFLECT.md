---
created_at: 2026-03-06T15:37:00
---

# Reflection - Build Artifact Pipelines And Docs

## Knowledge

- [1vyeN0000](../../knowledge/1vyeN0000.md) Build Firecracker Rootfs Images Without Privileged Mounts

## Observations

- The cleanest kernel pipeline was a pinned Firecracker CI kernel fetch with architecture-specific digests rather than trying to compile a kernel from source inside the MVP slice.
- Building the guest image in-repo was practical without privileged mounts once the pipeline staged BusyBox, `port-guest-agent`, and their runtime libraries into a directory and let `mkfs.ext4 -d` materialize the filesystem.
- Running `port machine launch` against the built artifacts was worth the extra verification step because it proved the artifact outputs were usable for the real launch path rather than just structurally valid on disk.

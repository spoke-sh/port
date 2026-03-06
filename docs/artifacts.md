# Artifact Contracts

Port ships one canonical artifact pipeline per artifact class for the MVP path.
The sample config in [`examples/port.toml`](../examples/port.toml) binds those
pipelines to `demo-kernel` and `demo-guest`.

## Kernel Artifact

- Canonical CLI:
  `port --config examples/port.toml artifacts build --artifact demo-kernel`
- Output path:
  `artifacts/kernel/demo/vmlinux`
- Source:
  pinned Firecracker CI kernel assets from the official `spec.ccfc.min` bucket
  for `v1.14`
- Current pinned keys:
  `firecracker-ci/v1.14/x86_64/vmlinux-6.1.155`
  `firecracker-ci/v1.14/aarch64/vmlinux-6.1.155`
- Validation:
  sha256 check against the pinned architecture-specific digest and a non-zero
  file size check

## Guest Image Artifact

- Canonical CLI:
  `port --config examples/port.toml artifacts build --artifact demo-guest`
- Output path:
  `artifacts/guest/demo/rootfs.ext4`
- Inputs:
  release build of `port-guest-agent`, BusyBox userspace from the development
  shell, runtime shared libraries discovered with `ldd`, and an `init` script
  authored in-repo
- Files installed into the image:
  `/init`
  `/bin/busybox`
  `/usr/bin/port-guest-agent`
  minimal `/etc/passwd` and `/etc/group`
- Validation:
  `e2fsck -fn` for filesystem integrity plus `debugfs` checks that `/init`,
  `/bin/busybox`, and `/usr/bin/port-guest-agent` exist and that `/init`
  launches `port-guest-agent`

## Operator Notes

- Run the artifact workflows from `nix develop` so `curl`, `busybox`,
  `mkfs.ext4`, `debugfs`, and related tooling are on `PATH`.
- Artifact paths are intentionally deterministic and derived from the checked-in
  model so later `port doctor` and `port machine launch` calls point at the
  same locations.

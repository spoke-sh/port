# Artifact Contracts

Port artifacts are now modeled as logical references plus concrete variants.
The sample config in [`examples/port.toml`](../examples/port.toml) binds that
contract to `demo-kernel` and `demo-guest`.

## Canonical Vocabulary

Each artifact now carries four related concepts:

- Reference:
  a logical identifier such as `demo-fs/port/demo-kernel:v1`
- Variant:
  one concrete selection of `architecture`, `substrate`, and
  `protection_mode`
- Distribution backend:
  where `push` publishes and `pull` fetches that variant
- Cache root:
  where Port stores a pulled or published local copy outside the canonical
  build output path

In the sample model:

- the shipped backend is `file-system`
- the sample store root is `artifact-store/demo-fs/`
- the sample cache root is `.port/cache/`
- reserved but not yet executable backends are `oci-registry` and `hosted-api`

## Canonical CLI

Build and validate one native variant:

```bash
port --config examples/port.toml artifacts build --artifact demo-kernel --architecture native
port --config examples/port.toml artifacts validate --artifact demo-kernel --architecture native
port --config examples/port.toml artifacts build --artifact demo-guest --architecture native
port --config examples/port.toml artifacts validate --artifact demo-guest --architecture native
```

Publish and fetch one selected variant:

```bash
port --config examples/port.toml artifacts push --artifact demo-kernel --architecture x86-64
rm -f artifacts/kernel/demo/x86_64/firecracker/standard/vmlinux
port --config examples/port.toml artifacts pull --artifact demo-kernel --architecture x86-64
```

The selector flags are the canonical compatibility surface:

- `--architecture <native|x86-64|aarch64>`
- `--substrate <firecracker|cloud-hypervisor|avf>`
- `--protection-mode <standard|pvm>`

Current runtime boundary:

- the demo build and validate pipelines only run for the native host
  architecture
- the sample config ships Firecracker/standard variants plus
  `x86_64/firecracker/pvm` kernel and guest-image variants
- `aarch64/firecracker/pvm` remains research-only and fails fast in the
  build/validate scripts
- push/pull are already variant-aware even when the selected variant is
  published or fetched on another host

## Sample Variant Layout

The sample config points at deterministic local output paths:

- kernel:
  `artifacts/kernel/demo/<architecture>/firecracker/standard/vmlinux`
- kernel PVM:
  `artifacts/kernel/demo/x86_64/firecracker/pvm/vmlinux`
- guest image:
  `artifacts/guest/demo/<architecture>/firecracker/standard/rootfs.ext4`
- guest image PVM:
  `artifacts/guest/demo/x86_64/firecracker/pvm/rootfs.ext4`

The file-backed store layout is similarly deterministic:

- store path:
  `artifact-store/demo-fs/<registry>/<repository>/<version>/<architecture>/<substrate>/<protection-mode>/<filename>`
- cache path:
  `.port/cache/<registry>/<repository>/<version>/<architecture>/<substrate>/<protection-mode>/<filename>`

For `demo-kernel` on `x86_64/firecracker/standard`, that becomes:

- local path:
  `artifacts/kernel/demo/x86_64/firecracker/standard/vmlinux`
- store path:
  `artifact-store/demo-fs/demo-fs/port/demo-kernel/v1/x86_64/firecracker/standard/vmlinux`
- cache path:
  `.port/cache/demo-fs/port/demo-kernel/v1/x86_64/firecracker/standard/vmlinux`

## Kernel Artifact

- Logical reference:
  `demo-fs/port/demo-kernel:v1`
- Source:
  pinned Firecracker CI kernel assets from the official `spec.ccfc.min` bucket
  for `v1.14`
- Current pinned keys:
  `firecracker-ci/v1.14/x86_64/vmlinux-6.1.155`
  `firecracker-ci/v1.14/aarch64/vmlinux-6.1.155`
- Validation:
  sha256 check against the pinned architecture-specific digest and a non-zero
  file size check
- PVM note:
  the current `x86_64/firecracker/pvm` kernel variant is materialized as its
  own selector/path and validation lane, but it is still seeded from the same
  pinned Firecracker CI demo kernel while the real PVM host-kit/runtime path is
  under construction

## Guest Image Artifact

- Logical reference:
  `demo-fs/port/demo-guest:v1`
- Inputs:
  release build of `port-guest-agent`, BusyBox userspace when available,
  runtime shared libraries discovered with `ldd`, and an `init` script authored
  in-repo
- Files installed into the image:
  `/init`
  `/bin/busybox`
  `/usr/bin/port-guest-agent`
  minimal `/etc/passwd` and `/etc/group`
- Validation:
  `e2fsck -fn` for filesystem integrity plus `debugfs` checks that `/init`,
  `/bin/busybox`, and `/usr/bin/port-guest-agent` exist and that `/init`
  launches `port-guest-agent`
- PVM note:
  the `x86_64/firecracker/pvm` guest-image variant carries explicit
  `/etc/port-protection-mode` and `/etc/port-guest-architecture` markers so
  validation can distinguish it from the standard Firecracker lane

## PVM Artifact Kit

Port's Firecracker/PVM lane is not a metadata-only variation of the standard
artifacts.

Required contract:

- kernel variant path under `.../x86_64/firecracker/pvm/`
- guest-image variant path under `.../x86_64/firecracker/pvm/`
- dedicated validation for those PVM variants
- no silent fallback to `standard` artifacts

The current foundation slice materializes those `x86_64` PVM selectors and
keeps `aarch64` as research-only. The full host-kit and artifact-kit contract
for that lane lives in
[`pvm.md`](pvm.md).

## Operator Notes

- Run the artifact workflows from the repository root so
  `examples/port.toml` resolves correctly.
- `nix develop` is one way to provide `curl`, `busybox`, `mkfs.ext4`,
  `debugfs`, and related tooling, but it is optional. Port also works when
  those tools are installed directly on the host.
- `push` and `pull` are the canonical artifact-mobility verbs even though the
  sample runtime currently implements only the file-backed backend.
- The file-backed backend is not the long-term hosted product story by itself;
  it is the shipped proof that Port's artifact vocabulary can span local build,
  publish, fetch, cache, and compatibility selection without changing the CLI.

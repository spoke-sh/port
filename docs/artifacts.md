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

- the shipped backends are `file-system` and `hosted-api`
- `oci-registry` is also executable when the selected variant points at a
  reachable OCI registry and `oras` is available on `PATH`
- the sample store root is `artifact-store/demo-fs/`
- the sample cache root is `.port/cache/`
- hosted control-plane storage lives under `.port/hosted/<control-plane>/artifacts/`

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

Local OCI registry proof:

1. Copy `examples/port.toml` to a temp config.
2. Replace `[artifacts.kernels.demo-kernel.reference] registry = "demo-fs"`
   with `registry = "127.0.0.1:5510"`.
3. Replace the existing `push` and `pull` sections with:

```toml
[artifacts.kernels.demo-kernel.distribution.push]
backend = "oci-registry"
transport = "plain-http"

[artifacts.kernels.demo-kernel.distribution.push.auth]
kind = "anonymous"

[artifacts.kernels.demo-kernel.distribution.pull]
backend = "oci-registry"
transport = "plain-http"

[artifacts.kernels.demo-kernel.distribution.pull.auth]
kind = "anonymous"
```

4. Start a local OCI registry. The repo-local proof tasks use Docker plus the
   standard `registry:2` image on `127.0.0.1:5510`.
5. Run the round trip:

```bash
just demo-push-oci
just demo-pull-oci
```

That proof:

- builds the native `demo-kernel` variant
- pushes it through `port artifacts push`
- removes the canonical local kernel path
- restores it through `port artifacts pull`
- never depends on a public registry

Publish and fetch the protected `x86_64/firecracker/pvm` variant:

```bash
port --config examples/port.toml artifacts push --artifact demo-kernel --architecture x86-64 --substrate firecracker --protection-mode pvm
rm -f artifacts/kernel/demo/x86_64/firecracker/pvm/vmlinux
port --config examples/port.toml artifacts pull --artifact demo-kernel --architecture x86-64 --substrate firecracker --protection-mode pvm
```

Hosted backend proof:

1. Copy `examples/port.toml` to a temp config.
2. Replace the existing `[control_planes.demo]`,
   `[artifacts.kernels.demo-kernel.distribution.push]`, and
   `[artifacts.kernels.demo-kernel.distribution.pull]` sections with:

```toml
[control_planes.demo]
endpoint = "http://127.0.0.1:7040"
audience = "port-hosted-demo"

[artifacts.kernels.demo-kernel.distribution.push]
backend = "hosted-api"
endpoint = "http://127.0.0.1:7040"

[artifacts.kernels.demo-kernel.distribution.pull]
backend = "hosted-api"
endpoint = "http://127.0.0.1:7040"
```

3. Run the hosted round trip through the canonical CLI:

```bash
cp examples/port.toml /tmp/port-hosted-artifacts.toml
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-hosted-artifacts.toml control-plane serve --control-plane demo --bind 127.0.0.1:7040
port --config /tmp/port-hosted-artifacts.toml artifacts build --artifact demo-kernel --architecture native
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-hosted-artifacts.toml artifacts push --artifact demo-kernel --architecture native
# On x86_64 remove artifacts/kernel/demo/x86_64/firecracker/standard/vmlinux.
# On aarch64 remove artifacts/kernel/demo/aarch64/firecracker/standard/vmlinux.
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-hosted-artifacts.toml artifacts pull --artifact demo-kernel --architecture native
```

The hosted control plane owns the stored bytes for that workflow. For
`demo-kernel`, Port persists the selected variant under:

- `.port/hosted/<control-plane>/artifacts/<registry>/<repository>/<version>/<architecture>/<substrate>/<protection-mode>/<filename>`

Auth expectations:

- the hosted artifact route uses the configured bearer token contract
- the sample control plane reads that token from `PORT_DEMO_TOKEN`
- the CLI sends it through the configured `authorization` header

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
  pinned upstream kernel sources for each compatibility lane:
  Firecracker CI demo kernels for the standard Firecracker lanes, and the
  sibling `pvm-builds` flake for the `x86_64/firecracker/pvm` lane
- Current pinned keys:
  `firecracker-ci/v1.14/x86_64/vmlinux-6.1.155`
  `firecracker-ci/v1.14/aarch64/vmlinux-6.1.155`
- Validation:
  compare the built artifact against the lane-specific upstream source plus a
  non-zero file size check
- PVM note:
  `x86_64/firecracker/pvm` now resolves a dedicated PVM guest kernel from
  `pvm-builds` via `PORT_PVM_BUILD_FLAKE_REF` or a sibling `../pvm-builds`
  checkout. It no longer reuses the standard Firecracker CI demo kernel.

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
  validation can distinguish it from the standard Firecracker lane, and
  `x86_64/firecracker/*` guest images now ship a sibling `initrd.cpio.gz` so
  Port can boot a read-only base image plus writable overlay on the hosted AWS
  PVM lane

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
- `nix develop` is one way to provide `curl`, `mkfs.ext4`, `debugfs`, and
  related tooling, and it exports `PORT_BUSYBOX_BIN` for workflows that need
  BusyBox. Port also works when those tools are installed directly on the host.
- `push` and `pull` are the canonical artifact-mobility verbs even though the
  shipped runtime now implements the file-backed, hosted-api, and oci-registry
  backends.
- The hosted control plane owns `.port/hosted/<control-plane>/artifacts/` for
  hosted pushes and pulls; the local caller still keeps the selected cache copy
  under `.port/cache/`.
- The file-backed backend is still useful for repo-local proofs, but it is no
  longer the only executable artifact distribution path.
- The repo-local OCI proof uses `oras` plus a local registry container; direct
  CLI use works the same way when those tools are installed on the host.

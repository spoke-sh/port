# Firecracker PVM Contract

Port keeps Firecracker/PVM in scope because it matters for cost-controlled
cloud execution, but it is not a drop-in switch on top of the current
Firecracker/KVM lane.

If you need the operator-facing AWS deployment narrative, start with
[`aws.md`](aws.md). This file is the lower-level contract for the host kit,
artifact kit, and hard boundaries behind that lane.

The implementation contract is narrower and more concrete:

- keep `x86_64` Firecracker/PVM as the first Port implementation lane
- treat it as a prepared host-kit plus artifact-kit problem
- keep `aarch64` Firecracker/PVM research-only until Port has a supportable
  Firecracker runtime path rather than only upstream kernel evidence

## Current Decision

| Architecture | Port decision | Why |
|--------------|---------------|-----|
| `x86_64` | Keep / hosted AWS prepared-node lane | Port now launches this lane through prepared hosted AWS nodes, but it still depends on a custom host kernel, a patched Firecracker build, `pti=off`, imported readiness, and dedicated guest images |
| `aarch64` | Research-only | Upstream arm64 protected-virtualization work is real, but Port does not yet have a vendor-grade Firecracker/PVM runtime contract to ship or validate |

Actuated's public product materials also matter to this decision:

- actuated clearly supports `arm64`, but on native Arm infrastructure and
  bare-metal-style microVM scheduling
- that is not the same claim as "Firecracker/PVM on arm64 cloud VMs is a
  supportable Port lane today"

Port therefore keeps the two ideas separate:

- native Arm microVM execution is useful and real
- arm64 Firecracker/PVM remains research-only until the Firecracker runtime
  story is supportable end-to-end

## x86_64 Host Kit Contract

The x86_64 Firecracker/PVM lane needs a dedicated host kit.

Required contract:

- Linux `x86_64` execution host
- custom host kernel booted specifically for the PVM-capable KVM lane
- host boot line includes `pti=off`
- patched Firecracker binary for the PVM lane
- the hosted Firecracker/PVM launch path must not fail on a missing `/dev/kvm`;
  the PVM host kit is the launch gate instead
- explicit operator separation between the standard Firecracker host kit and
  the PVM host kit

Port should treat those requirements as blocking, not advisory. If the PVM host
kit is absent, `port doctor` and any future PVM launch flow should fail fast.

## First-Class Nix Host Kit Surface

Port now exports the x86_64 AWS host-kit contract as first-class Nix surfaces:

- `nixosModules.aws-pvm-host`
- `packages.x86_64-linux.firecracker-pvm-host-kit`

That companion package carries the canonical downstream handoff assets:

- `bin/firecracker-pvm`
- `share/port/aws-pvm-host-kit.json`
- `share/port/nixos/aws-pvm-host.nix`

By default, that package ships Port's pinned `loopholelabs/firecracker`
no-KVM PVM build for `x86_64-linux`, not a renamed stock Firecracker binary.

The NixOS module sets the Port-owned host contract directly:

- boot args include `pti=off`
- `PORT_PVM_FIRECRACKER_BINARY` resolves to the canonical
  `firecracker-pvm` path surface
- `/etc/port/aws-pvm-host-kit.json` records the host-kit identity expected by
  `prepare-pvm-node`

If an image pipeline already carries concrete PVM-capable kernel or Firecracker
derivations, override `port.awsPvmHost.kernelPackages` and
`port.awsPvmHost.firecrackerPackage` there. Do not fork the module into a
downstream repo just to restate Port's host-kit contract.

The exported module uses a buildable Linux 6.12 fallback kernel by default so
downstream NixOS image builds do not fail before the real PVM kernel is wired
in. That fallback is only the kernel-side integration seam. The Firecracker
side now defaults to Port's pinned loopholelabs PVM build, while production
images still need the concrete patched kernel to be supplied through the
override point.

## x86_64 Artifact Kit Contract

The PVM lane also needs dedicated artifacts.

Required contract:

- kernel variant selected as `x86_64/firecracker/pvm`
- guest-image variant selected as `x86_64/firecracker/pvm`
- sibling initrd for the `x86_64/firecracker/pvm` guest image so Port can boot
  a read-only base rootfs with a writable overlay drive
- no reuse of the current `standard` Firecracker kernel or guest image
- the PVM kernel variant must resolve from a dedicated guest-kernel build
  source such as `pvm-builds`, not from the stock Firecracker CI kernel
- variant-specific validation instead of reusing the standard lane's checks

That keeps the artifact story honest: PVM is a separate compatibility lane with
its own build, pull, cache, and validation lifecycle.

## Validation Contract

Port's x86_64 PVM lane should only be considered honest when validation checks
all of the following:

1. Host architecture is Linux `x86_64`.
2. The host is booted into the PVM-capable kernel and the boot line contains
   `pti=off`.
3. The selected Firecracker binary is the patched PVM build.
4. PVM kernel and guest-image variants exist and pass variant-specific
   validation.
5. A real prepared host can boot a Firecracker/PVM guest as the final runtime
   proof.

Those checks are the contract that `port doctor`, artifact validation, and live
AWS hosted PVM proofs should continue to make explicit.

## Repository-Local Workflow

This workflow is intentionally narrower than the full hosted AWS deployment
story. It gives operators a reproducible way to prove the model, doctor, and
artifact contracts locally before they move to the hosted prepared-node lane:

```bash
port --config examples/port.toml doctor
port --config examples/port.toml artifacts build --artifact demo-kernel --architecture x86-64 --substrate firecracker --protection-mode pvm
port --config examples/port.toml artifacts validate --artifact demo-kernel --architecture x86-64 --substrate firecracker --protection-mode pvm
port --config examples/port.toml artifacts build --artifact demo-guest --architecture x86-64 --substrate firecracker --protection-mode pvm
port --config examples/port.toml artifacts validate --artifact demo-guest --architecture x86-64 --substrate firecracker --protection-mode pvm
port --config examples/port.toml artifacts push --artifact demo-kernel --architecture x86-64 --substrate firecracker --protection-mode pvm
port --config examples/port.toml artifacts push --artifact demo-guest --architecture x86-64 --substrate firecracker --protection-mode pvm
port --config examples/port.toml artifacts pull --artifact demo-kernel --architecture x86-64 --substrate firecracker --protection-mode pvm
port --config examples/port.toml artifacts pull --artifact demo-guest --architecture x86-64 --substrate firecracker --protection-mode pvm
```

What those commands prove today:

- the model resolves dedicated `x86_64/firecracker/pvm` kernel and guest-image
  variants
- the artifact pipelines materialize and validate those variants without
  silently reusing the standard Firecracker paths
- `port doctor` reports the `pvm:local:x86_64:*` host-kit checks for Linux
  platform, `x86_64` architecture, `pti=off`, and the patched
  `firecracker-pvm` binary contract
- a local PVM launch still blocks until the prepared x86_64 Linux host really
  satisfies that host-kit contract

The same operator workflow should also leave the standard Firecracker lane
usable. Building or validating `x86_64/firecracker/pvm` artifacts does not
replace the standard `x86_64/firecracker/standard` artifacts or their paths.

## AWS Hosted Prepared-Node Workflow

This is the hosted runtime contract behind [`aws.md`](aws.md). It keeps
placement, host-kit readiness, and live launch explicit through the hosted
control plane.

Runnable repo-local proof:

```bash
bash scripts/hosted-pvm-demo.sh
```

Human-reviewable artifact:

```bash
./scripts/render-hosted-pvm-proof.sh .keel/stories/VFgcoUoUd/EVIDENCE
```

Start from a copy of `examples/port.toml` and make these temporary changes:

- point `[control_planes.demo].endpoint` at `http://127.0.0.1:7040`
- switch `machines.cloud-aws.protection_mode` to `pvm`
- switch `machines.cloud-aws.rootfs_read_only` to `true` and add
  `[machines.cloud-aws.rootfs_overlay] size_mib = 16384`
- point the `x86_64/firecracker/pvm` kernel and guest-image variants at the
  prepared artifact paths available on the AWS node host
- export `PORT_PVM_FIRECRACKER_BINARY` to the patched `firecracker-pvm` binary
  on that prepared AWS node

```bash
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-pvm.toml control-plane serve --control-plane demo --bind 127.0.0.1:7040
PORT_PVM_FIRECRACKER_BINARY=/path/to/firecracker-pvm PORT_DEMO_TOKEN=demo-token port --config /tmp/port-pvm.toml node-agent serve --node aws-linux-node --bind 127.0.0.1:9234 --token node-secret
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-pvm.toml control-plane prepare-pvm-node --control-plane demo --node aws-linux-node --architecture x86-64 --provenance repo-proof --package-name firecracker-pvm-host-kit --package-version 2026.04 --host-kernel-release 6.12.0-port-pvm --firecracker-build v1.13.0-dev+loopholelabs.pvm.7f6c070fa09c
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-pvm.toml machine launch --machine cloud-aws
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-pvm.toml machine status --machine cloud-aws
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-pvm.toml machine stop --machine cloud-aws
```

Interpret those sample hosts this way:

- `cloud-generic` on `generic-linux-node` stays the denial-only proof for an
  unprepared hosted PVM lane. It remains useful because Port reports the
  machine as `malformed` with placement detail when the node only advertises a
  `planned` PVM lane rather than `ready`.
- `cloud-aws` on `aws-linux-node` is the canonical provider-backed hosted PVM
  contract. The checked-in sample inventory keeps AWS explicit so the host-kit,
  imported readiness, and failure surfaces remain provider-aware.
- `control-plane prepare-pvm-node` writes the imported ready record under
  `.port/hosted/<control-plane>/imported-inventory.json`; that imported record
  is the canonical repo-local proof that the prepared AWS node is advertising a
  ready hosted PVM lane.
- once you provide the prepared artifact paths plus `firecracker-pvm`, Port
  accepts placement because `aws-linux-node` advertises or imports a `ready`
  x86_64 PVM lane and then launches `cloud-aws` through the live control-plane
  and node-agent path.
- missing `firecracker-pvm`, missing host boot prerequisites, or missing PVM
  artifact paths fail explicitly for `cloud-aws`; Port does not silently fall
  back to the standard Firecracker lane.
- `aarch64/firecracker/pvm` stays research-only; there is no supported
  `prepare-pvm-node` or launch proof for that architecture, and the current
  hosted contract does not claim the AWS lane for GCP or Azure.

## Preserved Standard Lane

The PVM workflow is additive. The standard Firecracker lane must stay usable
while PVM work continues:

```bash
port --config examples/port.toml artifacts build --artifact demo-kernel --architecture x86-64 --substrate firecracker --protection-mode standard
port --config examples/port.toml artifacts build --artifact demo-guest --architecture x86-64 --substrate firecracker --protection-mode standard
port --config examples/port.toml machine launch --machine demo
```

Those commands prove that:

- standard Firecracker artifacts remain separate from the PVM artifact kit
- the shipped local Linux launch lane is still `standard`, not PVM
- PVM admission failures must never silently fall back to the standard lane

## arm64 Boundary

Port keeps the arm64 decision explicit:

- upstream arm64 work around protected virtualization, pKVM, and protected
  guest memory is relevant research input
- that upstream activity does not yet equal a shippable Firecracker/PVM lane
  for Port
- Port will not claim arm64 Firecracker/PVM support in the CLI, model,
  artifacts, or docs until the host kit, VMM path, artifact kit, and runtime
  proof all exist

This is a hard product boundary, not a soft maybe.

## Follow-On Work

The implementation order after this contract is:

1. Extend hosted launch beyond the prepared PVM lane to broader scheduler and
   provider-backed runtime rollout.

## Research Basis

- SlicerVM PVM docs: <https://docs.slicervm.com/tasks/pvm/>
- Actuated docs and product pages:
  <https://docs.actuated.com/>
  <https://docs.actuated.com/test-build/>
  <https://actuated.com/pricing>
- Alex Ellis on running Firecracker without nested KVM:
  <https://blog.alexellis.io/how-to-run-firecracker-without-kvm-on-regular-cloud-vms/>
- LWN coverage of x86 PVM and arm64 protected virtualization:
  <https://lwn.net/Articles/963718/>
  <https://lwn.net/Articles/1040628/>
- Kernel lore reference thread:
  <https://lore.kernel.org/lkml/CABgObfaSGOt4AKRF5WEJt2fGMj_hLXd7J2x2etce2ymvT4HkpA@mail.gmail.com/T/>

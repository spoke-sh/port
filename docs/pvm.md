# Firecracker PVM Contract

Port keeps Firecracker/PVM in scope because it matters for cost-controlled
cloud execution, but it is not a drop-in switch on top of the current
Firecracker/KVM lane.

The implementation contract is narrower and more concrete:

- keep `x86_64` Firecracker/PVM as the first Port implementation lane
- treat it as a prepared host-kit plus artifact-kit problem
- keep `aarch64` Firecracker/PVM research-only until Port has a supportable
  Firecracker runtime path rather than only upstream kernel evidence

## Current Decision

| Architecture | Port decision | Why |
|--------------|---------------|-----|
| `x86_64` | Keep / planned | Current public Firecracker/PVM operator docs still describe an x86_64-only lane with a custom host kernel, a patched Firecracker build, `pti=off`, and dedicated guest images |
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
- explicit operator separation between the standard Firecracker host kit and
  the PVM host kit

Port should treat those requirements as blocking, not advisory. If the PVM host
kit is absent, `port doctor` and any future PVM launch flow should fail fast.

## x86_64 Artifact Kit Contract

The PVM lane also needs dedicated artifacts.

Required contract:

- kernel variant selected as `x86_64/firecracker/pvm`
- guest-image variant selected as `x86_64/firecracker/pvm`
- no reuse of the current `standard` Firecracker kernel or guest image
- variant-specific validation instead of reusing the standard lane's checks

That keeps the artifact story honest: PVM is a separate compatibility lane with
its own build, pull, cache, and validation lifecycle.

## Validation Expectations

Future Port validation for the x86_64 PVM lane should check all of the
following:

1. Host architecture is Linux `x86_64`.
2. The host is booted into the PVM-capable kernel and the boot line contains
   `pti=off`.
3. The selected Firecracker binary is the patched PVM build.
4. PVM kernel and guest-image variants exist and pass variant-specific
   validation.
5. A real prepared host can boot a Firecracker/PVM guest as the final runtime
   proof.

Those checks belong in the future `port doctor` and artifact validation paths.

## Repository-Local Workflow

The current foundation slice is intentionally narrower than a real PVM launch.
It gives operators a reproducible way to prove the model, doctor, and artifact
contracts locally:

```bash
port --config examples/port.toml doctor
port --config examples/port.toml artifacts build --artifact demo-kernel --architecture x86-64 --substrate firecracker --protection-mode pvm
port --config examples/port.toml artifacts validate --artifact demo-kernel --architecture x86-64 --substrate firecracker --protection-mode pvm
port --config examples/port.toml artifacts build --artifact demo-guest --architecture x86-64 --substrate firecracker --protection-mode pvm
port --config examples/port.toml artifacts validate --artifact demo-guest --architecture x86-64 --substrate firecracker --protection-mode pvm
```

What those commands prove today:

- the model resolves dedicated `x86_64/firecracker/pvm` kernel and guest-image
  variants
- the artifact pipelines materialize and validate those variants without
  silently reusing the standard Firecracker paths
- `port doctor` reports the `pvm:local:x86_64:*` host-kit checks for Linux
  platform, `x86_64` architecture, `pti=off`, and the patched
  `firecracker-pvm` binary contract
- the current launch path still blocks on those host-kit checks because the
  real Firecracker/PVM runtime lane is not shipped yet

The same operator workflow should also leave the standard Firecracker lane
usable. Building or validating `x86_64/firecracker/pvm` artifacts does not
replace the standard `x86_64/firecracker/standard` artifacts or their paths.

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

1. Build and package the x86_64 PVM host kit.
2. Extend `port doctor` with explicit PVM host-kit checks.
3. Add x86_64/firecracker/pvm kernel and guest-image pipelines plus validation.
4. Add a Firecracker/PVM driver path that selects the host kit and PVM
   artifacts explicitly.
5. Teach the hosted/node-agent lane how to place workloads only on hosts that
   advertise the PVM host kit.

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

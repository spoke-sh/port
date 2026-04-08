# AWS Production Contract

Port's clearest production-oriented cloud narrative today is the hosted AWS
`x86_64` Firecracker/PVM lane, not the broader hosted standard demo lane.

That contract is intentionally narrow:

- canonical machine: `cloud-aws`
- canonical node: `aws-linux-node`
- canonical lane: `x86_64` + `firecracker` + `pvm`
- canonical readiness step: `port control-plane prepare-pvm-node`
- canonical lifecycle: `port machine launch`, `port machine status`, and
  `port machine stop`

If you need one document to answer "what is Port's strongest AWS deployment
story?", start here.

## Choose The Lane Deliberately

Port now has two different AWS-shaped hosted narratives, and they should not be
treated as equivalent.

| AWS lane | What it is for | What must be true | What it is not |
|----------|----------------|-------------------|----------------|
| Hosted Firecracker `standard` | Proving the live control-plane/node-agent split, the hosted guest/service contract, and repo-local application proofs | a registered hosted node, standard artifacts, hosted control plane, node agent | the strongest production-oriented cloud contract |
| Hosted Firecracker `pvm` on `x86_64` | Hardening a provider-specific AWS deployment path around a prepared node and no-fallback runtime contract | prepared AWS host kit, dedicated PVM artifacts, imported ready PVM lane, hosted control plane, node agent | EC2 provisioning, IAM, VPC, DNS, or a generic multi-provider promise |

The `standard` lane still matters. It proves the hosted control-plane split and
keeps the guest/service model honest. The PVM lane matters because it is the
place where Port's AWS story becomes specific enough to read like production
infrastructure instead of a generic hosted demo.

## What Must Be True

### 1. Hosted Control Plane Contract

Port still uses the same hosted split:

- a named control plane owns hosted lifecycle intent and inventory
- a node agent owns the runtime root and hypervisor processes on the AWS host
- the CLI and SDK keep the same `machine`, `guest`, and `service` verbs

In a repo-local proof those roles run via `port control-plane serve` and
`port node-agent serve`. In a deployed environment those roles would run
persistently, but the Port contract stays the same.

### 2. AWS Host Kit Contract

The AWS node must satisfy the PVM host kit before the lane is honest:

- Linux `x86_64`
- custom PVM-capable host kernel
- host boot line includes `pti=off`
- patched `firecracker-pvm` binary
- imported readiness advertising the node as `ready` for the AWS PVM lane
- no `/dev/kvm` launch requirement in the hosted Firecracker/PVM path; the PVM
  host-kit contract is the gate instead

Port should treat those as blocking requirements, not hints.

### AWS PVM Host Versus K3s Node

The AWS PVM host is the execution host in Port's hosted model. It is not the
same thing as a K3s node in the hosted K3s contract.

- AWS PVM host: prepared `x86_64` execution host running the Port node agent
  and hypervisor ownership
- K3s node: Firecracker guest microVM launched by Port on top of that prepared
  host

That distinction matters for HA. "Three K3s nodes" only becomes real HA when
those control-plane microVMs are spread across distinct prepared AWS hosts and
fronted by one stable HTTPS API endpoint.

### 3. Port-Owned Nix Host Kit Export

Port now exports the AWS `x86_64` PVM host contract directly from the Port
flake:

- `port.nixosModules.aws-pvm-host`
- `port.packages.x86_64-linux.firecracker-pvm-host-kit`

Use that module as the supported downstream source of truth for AMI builds.
The companion package publishes:

- `bin/firecracker-pvm`
- `share/port/aws-pvm-host-kit.json`
- `share/port/nixos/aws-pvm-host.nix`

By default, `bin/firecracker-pvm` is backed by Port's pinned
`loopholelabs/firecracker` no-KVM PVM release for `x86_64-linux`.

Minimal downstream import:

```nix
{
  inputs.port.url = "github:spoke-sh/port";

  outputs = { nixpkgs, port, ... }: {
    nixosConfigurations.aws-pvm-host = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        port.nixosModules.aws-pvm-host
        {
          system.stateVersion = "24.11";

          # Replace these with the concrete PVM-capable builds used for the
          # production image when they intentionally differ from Port's pinned
          # loopholelabs Firecracker/PVM package.
          # port.awsPvmHost.kernelPackages = pkgs.linuxPackagesFor myPortPvmKernel;
          # port.awsPvmHost.firecrackerPackage = myFirecrackerPvm;
        }
      ];
    };
  };
}
```

The module owns the Port contract: the expected kernel release identity,
`pti=off`, the canonical `firecracker-pvm` env surface, and the on-host manifest
path. If your image pipeline already carries the concrete patched kernel or VMM
derivations, override `port.awsPvmHost.kernelPackages` and
`port.awsPvmHost.firecrackerPackage` rather than cloning the module into a
downstream repo.

The default `kernelPackages` value is intentionally a buildable Linux 6.12
fallback so downstream AMI builds evaluate and realize cleanly. Treat that as
the wiring seam, not as the final production PVM kernel. Production images
should still override the kernel input, and only override the Firecracker
package when they intentionally ship a different provenanced PVM build.

### 4. AWS Artifact Kit Contract

The AWS PVM lane also needs dedicated guest artifacts:

- kernel selector: `x86_64/firecracker/pvm`
- guest-image selector: `x86_64/firecracker/pvm`
- sibling `initrd.cpio.gz` for that guest image so Port can boot a read-only
  base rootfs with a writable overlay drive
- no reuse of the `standard` Firecracker artifact variants

This is the operational reason the AWS PVM lane is stronger than the generic
hosted standard lane: the runtime, the host kit, and the artifacts are all
explicitly bound together.

### 5. Canonical Machine And Node Identities

The checked-in sample already names the production-oriented AWS path:

- host: `aws-linux`
- node: `aws-linux-node`
- machine: `cloud-aws`
- host group: `aws-builders`

Keep those identities explicit in the docs and in operator workflows. The
generic hosted node remains useful as a denial path, not as the canonical AWS
surface.

## Canonical Workflow

### Downstream AMI Handoff

The supported downstream seam is now "import Port as a flake input", not
"author a repo-local replacement module":

```nix
{
  inputs.port.url = "github:spoke-sh/port";
}
```

Downstream `infra` can then build its AMI against the exported
`port.nixosModules.aws-pvm-host` surface directly:

```bash
infra image --env prod build-pvm-ami
```

When you need to test a local Port checkout before pushing it, override the
flake input instead of threading a second module path:

```bash
nix build .#aws-pvm-amazon-image \
  --override-input port path:/absolute/path/to/port
```

That keeps responsibility split cleanly:

- Port owns the AWS PVM host-kit module, manifest, boot-line contract, and
  canonical `firecracker-pvm` surface.
- downstream `infra` owns AMI build, upload, VM Import/Export, publication, and
  later consumption of the resulting `ami-...` ID.

Start from a copy of `examples/port.toml` and harden only the AWS path:

1. Keep `[control_planes.demo]` pointed at the hosted control plane you will
   use.
2. Switch `[machines.cloud-aws].protection_mode` to `pvm`.
3. Switch `[machines.cloud-aws].architecture` to `x86_64` when the deployment
   config should be explicit instead of `native`.
4. Switch `[machines.cloud-aws].rootfs_read_only` to `true` and add
   `[machines.cloud-aws.rootfs_overlay] size_mib = 16384` so the guest boots a
   read-only base image with a writable overlay instead of materializing a full
   writable copy on each launch.
5. Point the `x86_64/firecracker/pvm` kernel and guest-image variants at the
   prepared artifact paths available to `aws-linux-node`.
6. Export `PORT_PVM_FIRECRACKER_BINARY` to the patched `firecracker-pvm`
   binary on the AWS execution host.

Canonical operator flow:

```bash
port --config /tmp/port-aws-pvm.toml doctor
port --config /tmp/port-aws-pvm.toml artifacts validate --artifact demo-kernel --architecture x86-64 --substrate firecracker --protection-mode pvm
port --config /tmp/port-aws-pvm.toml artifacts validate --artifact demo-guest --architecture x86-64 --substrate firecracker --protection-mode pvm

PORT_DEMO_TOKEN=demo-token port --config /tmp/port-aws-pvm.toml control-plane serve --control-plane demo --bind 127.0.0.1:7040
PORT_PVM_FIRECRACKER_BINARY=/path/to/firecracker-pvm PORT_DEMO_TOKEN=demo-token port --config /tmp/port-aws-pvm.toml node-agent serve --node aws-linux-node --bind 127.0.0.1:9234 --token node-secret

PORT_DEMO_TOKEN=demo-token port --config /tmp/port-aws-pvm.toml control-plane prepare-pvm-node --control-plane demo --node aws-linux-node --architecture x86-64 --provenance repo-proof --package-name firecracker-pvm-host-kit --package-version 2026.04 --host-kernel-release 6.12.0-port-pvm --firecracker-build v1.13.0-dev+loopholelabs.pvm.7f6c070fa09c

PORT_DEMO_TOKEN=demo-token port --config /tmp/port-aws-pvm.toml machine launch --machine cloud-aws
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-aws-pvm.toml machine status --machine cloud-aws
PORT_DEMO_TOKEN=demo-token port --config /tmp/port-aws-pvm.toml machine stop --machine cloud-aws
```

If you are building hosted K3s on top of the AWS PVM lane, keep the layer split
explicit:

- use prepared AWS PVM hosts as the execution fleet
- launch the K3s control-plane and worker nodes as guest microVMs on that fleet
- keep those guest microVMs on a read-only base image plus writable overlay so
  launch retries stay idempotent and do not recopy the full guest rootfs
- front the control-plane microVMs with an external load balancer or VIP and
  publish that stable HTTPS address as `api_endpoint` in `[k3s_clusters.*]`
- set `control_plane_scheduler = "spread"` so new control-plane microVM
  placements fail instead of reusing an already occupied prepared host
- do not count "multiple microVMs on one prepared host" as real HA

Interpret those commands this way:

- `port doctor` validates that the selected config is trying to use the PVM
  lane rather than silently drifting back to `standard`
- artifact validation proves the required PVM variants exist before launch
- `prepare-pvm-node` is the readiness bridge between a planned node and a real
  prepared AWS host
- `machine launch/status/stop` remain the canonical operator verbs even though
  the ownership model is hosted

## Failure Surfaces Must Stay Honest

The AWS PVM path is only useful if failures stay provider-aware and explicit.

Expect these to fail fast:

- missing imported readiness for `aws-linux-node`
- missing PVM host kernel or missing `pti=off`
- missing patched `firecracker-pvm`
- missing `x86_64/firecracker/pvm` kernel or guest-image variants
- attempts to treat `cloud-generic`, GCP, or Azure as equivalent to the AWS PVM
  contract

What must never happen:

- silent fallback from AWS `pvm` to AWS `standard`
- silent fallback from AWS to generic hosted placement
- accidental documentation drift that turns arm64 Firecracker/PVM into an
  implied promise

## Repo-Local Proof Versus Production Contract

Port's repo-local proof still matters, but it is not the whole production
story.

Repo-local proof gives you:

- the real command family
- the real lane identities
- the real readiness import step
- human-reviewable evidence that `cloud-aws` launches through the hosted PVM
  path

It does not mean Port already ships:

- EC2 instance provisioning
- IAM setup
- VPC or load-balancer automation
- DNS management
- downstream GitOps or application rollout policy

Port owns the runtime contract. The surrounding AWS platform is still explicit
follow-on scope.

## Boundaries To Keep Explicit

- AWS hosted PVM is `x86_64` only today
- arm64 Firecracker/PVM remains research-only
- GCP and Azure do not inherit the AWS PVM contract
- hosted Firecracker `standard` still exists and remains useful as the simpler
  hosted proof path

## Related Docs

- [`README.md`](../README.md) for the top-level product posture and doc map
- [`CONFIGURATION.md`](../CONFIGURATION.md) for the config shape behind this lane
- [`docs/hosted.md`](hosted.md) for control-plane and node-agent ownership
- [`docs/cloud.md`](cloud.md) for the wider provider and lane matrix
- [`docs/pvm.md`](pvm.md) for the low-level host-kit and artifact-kit contract

---
id: 1vydg7000
---

# Cloud Linux and PVM Viability — Evidence

## Market Research

### Existing Solutions

Current Firecracker adopters typically assume a Linux host they already control.
Cloud-hosted variants usually rely on bare metal or nested virtualization
instead of a distinct "managed Firecracker" control plane. That means Port's
best differentiator is a unified operator experience across local and remote
Linux hosts, not a cloud-specific API from day one.

### Competitive Landscape

Comparable tools and projects tend to split into two camps:

- local Firecracker wrappers that assume Linux-only operators; and
- higher-level platforms that hide the virtualization substrate but do not
  expose a consistent guest-operation model for CLI users.

That gap supports a Port MVP that keeps the CLI/model canonical while treating
cloud execution as a host-targeting extension of the same workflow.

### Market Size

The opportunity is limited less by total VM demand than by the number of teams
that need stronger isolation than containers while still wanting an operator-
friendly local and cloud workflow. This favors a narrow, credible MVP over a
provider-wide matrix with weak support.

## Technical Research

### Feasibility

Cloud feasibility depends on KVM or an equivalent nested-virtualization story:

- AWS documents nested virtualization support on selected EC2 instance families,
  including `c8i`, `m8i`, and `r8i`, which makes an AWS nested-host lane
  technically plausible for MVP partial implementation.
- Google Cloud documents nested virtualization on selected Intel machine
  families, making a remote Linux host lane plausible there as well.
- Azure's official nested-virtualization guidance centers on Hyper-V, and
  Microsoft explicitly states that non-Microsoft virtualization on Hyper-V is
  not supported. Azure's confidential VM FAQ also says nested virtualization is
  not supported.

This makes "remote Linux host" a credible abstraction for AWS and GCP, but not
yet a supportable Azure Firecracker lane for MVP.

### Prior Art

We can build on:

- Firecracker's own requirement that operators provide a Linux host, kernel, and
  rootfs and then configure networking and boot through the API or wrapper.
- Existing cloud-provider nested-virtualization docs rather than inventing a
  cloud-specific substrate.
- Port's planned shared model, which can describe both `local` and `remote`
  Linux hosts with one machine/artifact contract.

### Proof of Concepts

No cloud proof-of-concept has been run in this repository yet. The immediate
implementation slice should therefore stop at host-profile modeling, CLI
targeting, and documentation unless runtime access to a supported cloud host is
available.

## User Research

### Target Users

Users who already understand Linux and virtualization but need one operator path
across laptops, workstations, and cloud Linux hosts benefit most. This includes
platform engineers, automation authors, and agent runtimes that need stronger
isolation than process sandboxes.

### Pain Points

They currently have to:

- learn different workflows for local experimentation versus cloud-hosted
  capacity;
- manually discover which hosts support nested virtualization; and
- piece together guest access, artifact production, and platform constraints
  from multiple tools and documents.

### Validation

Validation comes from the MVP acceptance criteria themselves: the product is not
considered complete unless cloud Linux support is designed, partially
implemented, and documented alongside macOS and Windows operator workflows.

## Key Findings

1. AWS now documents nested virtualization on selected EC2 families such as
   `c8i`, `m8i`, and `r8i`, which is enough to justify an AWS-oriented partial
   implementation for remote Linux hosts.
2. Google Cloud documents nested virtualization on selected Intel machine
   families, so a generic remote-Linux host model can also describe a GCP lane.
3. Azure does not currently provide a supportable Firecracker MVP lane: official
   docs focus on Hyper-V, non-Microsoft virtualization is not supported there,
   and Azure confidential VMs explicitly do not support nested virtualization.
4. The PVM lane should be dropped from MVP. Current provider support does not
   justify it: AWS nested virtualization support is documented on Intel-based
   families while AWS AMD SEV-SNP support is documented on different families,
   and Azure confidential VMs do not support nested virtualization.

## Unknowns

- The performance and cost overhead of nested virtualization on the currently
  supported cloud families.
- Whether GCP's nested-virtualization matrix is broad enough for a first live
  cloud proof once runtime work reaches that stage.
- Which host networking assumptions can remain portable between local Linux,
  AWS, and GCP without a second runtime design.

## Sources

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | web | manual:web-search | https://github.com/firecracker-microvm/firecracker/blob/main/docs/getting-started.md | 2026-03-06 | 2026-03-06 | high | medium | Firecracker setup assumes a Linux host and validates the remote-Linux-host framing. |
| SRC-02 | web | manual:web-search | https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/amazon-ec2-nested-virtualization.html | 2026-03-06 | 2026-03-06 | high | high | AWS documents nested virtualization on selected EC2 families, supporting a limited Firecracker-host lane. |
| SRC-03 | web | manual:web-search | https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/snp-requirements.html | 2026-03-06 | 2026-03-06 | high | high | AWS SEV-SNP support lands on a different lane than nested virtualization, which helps justify dropping MVP PVM. |
| SRC-04 | web | manual:web-search | https://cloud.google.com/compute/docs/instances/nested-virtualization/overview | 2026-03-06 | 2026-03-06 | high | high | Google Cloud documents nested virtualization on selected Intel families, supporting a second remote-Linux-host path. |
| SRC-05 | web | manual:web-search | https://learn.microsoft.com/en-us/troubleshoot/windows-server/high-availability/hyper-v-nested-virtualization | 2026-03-06 | 2026-03-06 | high | high | Microsoft's support policy limits Azure to Hyper-V-centric nesting and does not support non-Microsoft virtualization. |
| SRC-06 | web | manual:web-search | https://learn.microsoft.com/en-us/azure/confidential-computing/confidential-vm-faq | 2026-03-06 | 2026-03-06 | high | high | Azure confidential VMs do not support nested virtualization, ruling out an MVP Firecracker lane there. |

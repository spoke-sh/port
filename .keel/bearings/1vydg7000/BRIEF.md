# Cloud Linux and PVM Viability — Brief

## Hypothesis

Port should treat cloud support as "remote Linux Firecracker hosts" managed
through the same CLI and model used for local Linux. A separate PVM
(protected/confidential VM) lane should only stay in MVP scope if current cloud
providers support nested virtualization in a way that can plausibly host
Firecracker without a different architecture.

## Problem Space

Firecracker requires Linux and KVM. Cloud providers expose nested
virtualization unevenly, and protected/confidential VM offerings often have
different hardware or hypervisor constraints. Port needs a support matrix that
is technically credible, teachable to operators, and small enough to partially
implement during MVP rather than promising every cloud path at once.

## Context

Port's initial MVP was still centered on local Linux Firecracker launch. Before
expanding the roadmap, the board needed a concrete answer on whether cloud
support should mean remote Linux hosts that still run Firecracker or whether
the product had to broaden into a different protected-VM or hosted
architecture.

## Objectives

- Determine which cloud providers currently expose a supportable nested-KVM
  lane for Firecracker-style remote Linux hosts.
- Decide whether a protected/confidential VM lane belongs in MVP scope or
  should be dropped.
- Turn the result into a concrete planning direction instead of a vague support
  matrix promise.

## Scope

- In scope: current AWS, GCP, and Azure support boundaries for nested
  virtualization and confidential/protected VM offerings; the effect of those
  boundaries on Port's MVP promise; and the next board slice to plan.
- Out of scope: live cloud runtime benchmarks, provider cost modeling, and the
  full hosted-control-plane design that later work may require.

## Success Criteria

- [x] A current cloud support matrix exists for local-style Firecracker hosts on AWS, GCP, and Azure.
- [x] The PVM lane has an explicit keep-or-drop decision backed by current provider documentation.
- [x] Research results are concrete enough to drive at least one cloud-oriented planning or implementation slice.

## Research Questions

- Which current cloud offerings can credibly host nested KVM workloads for
  Firecracker?
- Is any protected/confidential VM lane currently compatible with a Firecracker
  MVP without separate architecture work?
- Which cloud path should be partially implemented first to preserve a single
  CLI/model story?

## Open Questions

- What performance and cost penalties will remain once Port moves from provider
  documentation to real host validation?
- How much provider-specific host networking or image preparation work will be
  needed beyond the first support matrix?

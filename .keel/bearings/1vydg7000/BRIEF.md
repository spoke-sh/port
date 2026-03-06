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

## Success Criteria

- [ ] A current cloud support matrix exists for local-style Firecracker hosts on AWS, GCP, and Azure.
- [ ] The PVM lane has an explicit keep-or-drop decision backed by current provider documentation.
- [ ] Research results are concrete enough to drive at least one cloud-oriented planning or implementation slice.

## Open Questions

- Which current cloud offerings can credibly host nested KVM workloads for Firecracker?
- Is any protected/confidential VM lane currently compatible with a Firecracker MVP without separate infrastructure work?
- Which cloud path should be partially implemented first to preserve a single CLI/model story?

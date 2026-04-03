---
# system-managed
id: VFgcpTciv
status: done
created_at: 2026-04-02T18:19:16
updated_at: 2026-04-02T19:07:22
# authored
title: Route Cloud Aws PVM Launch Through Prepared AWS Node
type: feat
operator-signal:
scope: VFgcPDfEj/VFgclbQzC
index: 2
started_at: 2026-04-02T18:56:49
submitted_at: 2026-04-02T19:07:18
completed_at: 2026-04-02T19:07:22
---

# Route Cloud Aws PVM Launch Through Prepared AWS Node

## Summary

Route canonical `cloud-aws` lifecycle commands through the live hosted
control-plane and node-agent path once an AWS node is prepared for the PVM
lane, and keep failures provider-aware when that readiness is missing.

## Acceptance Criteria

<!-- verify: manual, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-01/AC-01] `port machine launch --machine cloud-aws`, `status`, and `stop` succeed through the live hosted control-plane and node-agent path when an x86_64 AWS node advertises ready PVM preparation. <!-- [SRS-01/AC-01] verify: manual, proof: ac-1.log -->
<!-- verify: manual, SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-02/AC-02] If the AWS hosted PVM lane is missing prerequisites or still planned, Port fails with actionable `cloud-aws` guidance and does not fall back to the standard Firecracker/KVM lane. <!-- [SRS-02/AC-02] verify: manual, proof: ac-2.log -->

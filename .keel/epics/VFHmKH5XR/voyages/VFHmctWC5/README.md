---
# system-managed
id: VFHmctWC5
status: done
epic: VFHmKH5XR
created_at: 2026-03-29T12:21:22
# authored
title: Replace Demo API With GitOps-Capable Local K3s Runtime
index: 1
updated_at: 2026-03-29T12:24:38
started_at: 2026-03-29T12:27:56
completed_at: 2026-03-29T17:14:22
---

# Replace Demo API With GitOps-Capable Local K3s Runtime

> Upgrade Port's local single-node cluster lane from a demo API to a real GitOps-capable K3s control plane that supports normal kubeconfig handoff, Kubernetes discovery, Flux install, and Helm operator install.

## Documents

<!-- BEGIN DOCUMENTS -->
| Document | Description |
|----------|-------------|
| [SRS.md](SRS.md) | Requirements and verification criteria |
| [SDD.md](SDD.md) | Architecture and implementation details |
| [VOYAGE_REPORT.md](VOYAGE_REPORT.md) | Narrative summary of implementation and evidence |
| [COMPLIANCE_REPORT.md](COMPLIANCE_REPORT.md) | Traceability matrix and verification proof |
<!-- END DOCUMENTS -->

## Stories

<!-- BEGIN GENERATED -->
**Progress:** 3/3 stories complete

| Title | Type | Status |
|-------|------|--------|
| [Replace Demo Local Cluster Stub With Real K3s Control Plane](../../../../stories/VFHn1OVki/README.md) | feat | done |
| [Harden Kubeconfig Handoff And Kubernetes Discovery](../../../../stories/VFHn1Ozkj/README.md) | feat | done |
| [Prove Flux And Pulumi Operator Install Against Port Kubeconfig](../../../../stories/VFHn1PHka/README.md) | feat | done |
<!-- END GENERATED -->

## Retrospective

**What went well:** Port now boots a real local K3s lane, hands off kubeconfig cleanly, and supports direct Flux and Helm client proof.

**What was harder than expected:** Downstream consumer verification was originally mixed into the Port closure path and had to be split out explicitly.

**What would you do differently:** Separate Port contract closure from downstream consumer verification earlier when scoping future missions.


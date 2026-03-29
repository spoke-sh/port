---
# system-managed
id: VFHn1Pslh
status: icebox
created_at: 2026-03-29T12:22:56
updated_at: 2026-03-29T16:30:22
# authored
title: Verify Unchanged Downstream Infra GitOps Handoff
type: feat
operator-signal:
scope: VFHmKH5XR/VFHmctWC5
index: 4
started_at: 2026-03-29T16:28:02
---

# Verify Unchanged Downstream Infra GitOps Handoff

## Summary

Verify that downstream `spoke infra` can consume Port's local cluster handoff
unchanged through its local bootstrap and health flows once the real K3s
runtime and GitOps prerequisites are in place.

## Acceptance Criteria

- [ ] [SRS-05/AC-01] Downstream `infra bootstrap --env local` succeeds unchanged against the Port-provided cluster handoff. <!-- verify: manual, SRS-05:start:end -->
- [ ] [SRS-NFR-03/AC-02] Downstream `infra health --env local` succeeds unchanged while this voyage remains explicitly local-only and single-node only. <!-- verify: manual, SRS-NFR-03:start:end -->

## Current Blocker

Unchanged downstream `infra` proof still stops on the consumer health probe, not
on Port bootstrap or GitOps install. `just bootstrap` succeeds unchanged, but
`just health` fails because `infra` looks for deployment
`pulumi-kubernetes-operator` while the Helm chart installs
`pulumi-kubernetes-operator-controller-manager`. See
`EVIDENCE/ac-1.infra-bootstrap.log`, `EVIDENCE/ac-2.infra-health.log`, and
`EVIDENCE/ac-2.infra-ps.log`.

---
# system-managed
id: VGcgtDfDT
status: done
created_at: 2026-04-12T16:39:11
updated_at: 2026-04-12T17:36:36
# authored
title: Enforce Managed Service Ownership For Hosted K3s
type: feat
operator-signal:
scope: VGcgU9T57/VGcghwZrb
index: 2
started_at: 2026-04-12T17:22:27
submitted_at: 2026-04-12T17:36:32
completed_at: 2026-04-12T17:36:36
---

# Enforce Managed Service Ownership For Hosted K3s

## Summary

Remove legacy detached hosted K3s paths from the valid runtime contract so
hosted workers and servers exist under managed Port service ownership only.

## Acceptance Criteria

- [x] [SRS-02/AC-01] Port rejects, replaces, or otherwise eliminates legacy detached hosted K3s paths in favor of managed-service ownership. <!-- verify: manual, SRS-02:start:end, proof: ac-1.log -->
- [x] [SRS-NFR-01/AC-02] Managed-service ownership remains explicit in runtime artifacts and service status after the cutover. <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-2.gif -->

## Proof

- AC-01: `EVIDENCE/ac-1.log` records the script inspection showing the hosted K3s proof now drives canonical `cluster up|status|kubeconfig|down` plus managed-service start/status/list expectations instead of detached guest-exec bootstrap shelling.
- AC-02: `EVIDENCE/ac-2.log`, `EVIDENCE/ac-2.gif`, and `EVIDENCE/hosted-k3s-workflow.cast` capture the human-reviewable hosted K3s workflow, including direct `service status` and `cluster status` surfaces that expose managed-service ownership and clear legacy-runtime drift.

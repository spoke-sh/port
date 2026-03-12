---
id: VDfF1dZM9
title: Introduce Canonical Volume And Attachment Model
type: feat
status: done
created_at: 2026-03-12T07:40:40
updated_at: 2026-03-12T07:56:01
operator-signal: 
scope: VDcStQqlo/VDfEyGkVf
index: 3
started_at: 2026-03-12T07:47:30
completed_at: 2026-03-12T07:56:01
---

# Introduce Canonical Volume And Attachment Model

## Summary

Add a canonical attached-volume contract to the Port model so machines can
declare non-root block-volume attachments without overloading the existing
`guest_image` rootfs artifact path.

## Acceptance Criteria

<!-- verify: command, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-01/AC-01] The model and config surfaces add explicit machine-level attached-volume declarations without replacing the current `guest_image` and `rootfs_read_only` contract. <!-- [SRS-01/AC-01] verify: cargo test -q -p port-model volume_attachment_contract, proof: ac-1.log -->
<!-- verify: command, SRS-NFR-02:start:end, proof: ac-2.log -->
- [x] [SRS-NFR-02/AC-02] Existing machines that declare no attachments preserve the current machine contract and validation behavior after the new model lands. <!-- [SRS-NFR-02/AC-02] verify: cargo test -q -p port-model machine_contract_without_attachments_regression, proof: ac-2.log -->

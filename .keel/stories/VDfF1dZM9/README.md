---
id: VDfF1dZM9
title: Introduce Canonical Volume And Attachment Model
type: feat
status: backlog
created_at: 2026-03-12T07:40:40
updated_at: 2026-03-12T07:44:09
operator-signal: 
scope: VDcStQqlo/VDfEyGkVf
index: 3
---

# Introduce Canonical Volume And Attachment Model

## Summary

Add a canonical attached-volume contract to the Port model so machines can
declare non-root block-volume attachments without overloading the existing
`guest_image` rootfs artifact path.

## Acceptance Criteria

<!-- verify: command, SRS-01:start:end, proof: ac-1.log -->
- [ ] [SRS-01/AC-01] The model and config surfaces add explicit machine-level attached-volume declarations without replacing the current `guest_image` and `rootfs_read_only` contract. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q volume_attachment_contract', proof: ac-1.log -->
<!-- verify: command, SRS-NFR-02:start:end, proof: ac-2.log -->
- [ ] [SRS-NFR-02/AC-02] Existing machines that declare no attachments preserve the current machine contract and validation behavior after the new model lands. <!-- [SRS-NFR-02/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q machine_contract_without_attachments_regression', proof: ac-2.log -->

---
# system-managed
id: VFDk8gGoC
status: done
created_at: 2026-03-28T19:46:24
updated_at: 2026-03-28T21:35:25
# authored
title: Stage Offline K3s Artifacts And Guest Profile
type: feat
operator-signal:
scope: VFDhlRjOf/VFDk8fdnG
index: 2
started_at: 2026-03-28T21:10:27
submitted_at: 2026-03-28T21:35:21
completed_at: 2026-03-28T21:35:25
---

# Stage Offline K3s Artifacts And Guest Profile

## Summary

Make the first local cluster bootstrap path Port-owned by staging K3s inputs and
the required guest runtime dependencies without relying on guest-side live
network fetches.

## Acceptance Criteria

<!-- verify: manual, SRS-02:start:end -->
- [x] [SRS-02/AC-01] The canonical local cluster bootstrap path uses Port-owned artifact staging or a kube-ready guest profile and does not rely on guest-side `curl https://get.k3s.io`. <!-- verify: manual, SRS-02, proof: ac-1.log-->
<!-- verify: manual, SRS-NFR-02:start:end -->
- [x] [SRS-NFR-02/AC-02] Repo-local verification proves the staged inputs and guest profile are sufficient for the first local bootstrap slice. <!-- verify: manual, SRS-NFR-02, proof: ac-2.log-->

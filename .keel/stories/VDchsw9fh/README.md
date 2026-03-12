---
id: VDchsw9fh
title: Surface AVF Distribution Boundary In Docs And Doctor
type: feat
status: done
created_at: 2026-03-11T21:16:29
updated_at: 2026-03-11T22:09:14
scope: VDcT0vaPb/VDchK6xzs
index: 2
started_at: 2026-03-11T22:04:41
completed_at: 2026-03-11T22:09:14
---

# Surface AVF Distribution Boundary In Docs And Doctor

## Summary

Align the install docs and doctor/help surfaces with the real AVF runtime
contract so macOS operators get explicit launcher-helper, entitlement, and
unsupported-host guidance as part of the packaged Port experience.

## Acceptance Criteria

<!-- verify: command, SRS-04:start:end, proof: ac-1.log -->
- [x] [SRS-04/AC-01] macOS release guidance and doctor/help surfaces explain the `PORT_AVF_LAUNCHER` requirement, the launcher-helper role, the distributed-target entitlement boundary, and the expected unsupported-host guidance. <!-- [SRS-04/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime avf && rg -n "PORT_AVF_LAUNCHER|entitlement|distributed" README.md RELEASE.md docs/avf.md', proof: ac-1.log -->
<!-- verify: command, SRS-05:start:end, proof: ac-2.log -->
- [x] [SRS-05/AC-02] The release checklist for the installable slice anchors validation on `just`, `port doctor`, workspace tests, package proof commands, and board health instead of a disconnected packaging-only checklist. <!-- [SRS-05/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "just mission|just doctor|just test|port doctor|package-proof|just package" README.md RELEASE.md justfile .justfiles', proof: ac-2.log -->
<!-- verify: command, SRS-NFR-02:start:end, proof: ac-3.log -->
- [x] [SRS-NFR-02/AC-03] AVF packaging guidance fails fast with explicit boundaries and does not introduce a second macOS-only operator workflow or fallback surface. <!-- [SRS-NFR-02/AC-03] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-cli machine_commands && rg -n "macOS-only|launcher-helper|fallback" docs/avf.md README.md RELEASE.md', proof: ac-3.log -->

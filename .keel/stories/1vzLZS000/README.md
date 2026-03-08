---
id: 1vzLZS000
title: Define Avf Machine Contract And Doctor Checks
type: feat
status: done
created_at: 2026-03-08T14:21:54
updated_at: 2026-03-08T14:42:17
scope: 1vzJKE000/1vzLYD000
started_at: 2026-03-08T14:24:27
completed_at: 2026-03-08T14:42:17
---

# Define Avf Machine Contract And Doctor Checks

## Summary

Define the macOS-only AVF machine-selection and doctor contract so Port can
identify valid AVF targets, reject unsupported hosts, and surface entitlement
or availability boundaries before runtime work lands.

## Acceptance Criteria

<!-- verify: command, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-01/AC-01] AVF-targeted machines validate as macOS-only `standard`-protection local machines and fail fast on non-macOS or AVF/PVM selections. <!-- [SRS-01/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzLZS000/verify-ac-1.sh, proof: ac-1.log -->
<!-- verify: command, SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-02/AC-02] `port doctor` surfaces AVF-focused macOS checks plus explicit AVF availability or entitlement boundaries through the canonical CLI output. <!-- [SRS-02/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzLZS000/verify-ac-2.sh, proof: ac-2.log -->

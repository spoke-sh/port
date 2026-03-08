---
id: 1vzLZS000
title: Define Avf Machine Contract And Doctor Checks
type: feat
status: backlog
created_at: 2026-03-08T14:21:54
updated_at: 2026-03-08T14:23:39
scope: 1vzJKE000/1vzLYD000
---

# Define Avf Machine Contract And Doctor Checks

## Summary

Define the macOS-only AVF machine-selection and doctor contract so Port can
identify valid AVF targets, reject unsupported hosts, and surface entitlement
or availability boundaries before runtime work lands.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] AVF-targeted machines validate as macOS-only
  `standard`-protection local machines and fail fast on non-macOS or AVF/PVM
  selections.
- [ ] [SRS-02/AC-02] `port doctor` surfaces AVF-focused macOS checks plus
  explicit AVF availability or entitlement boundaries through the canonical
  CLI output.

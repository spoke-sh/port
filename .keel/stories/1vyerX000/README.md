---
id: 1vyerX000
title: Publish Cloud Support Matrix
type: feat
status: in-progress
created_at: 2026-03-06T15:45:43
updated_at: 2026-03-06T15:58:23
scope: 1vydg7000/1vyeq5000
started_at: 2026-03-06T15:58:23
---

# Publish Cloud Support Matrix

## Summary

Publish the remote Linux support matrix, provider boundaries, operator workflow,
and explicit PVM drop decision in the README and supporting docs so the cloud
lane is discoverable and honest at the CLI product surface.

## Acceptance Criteria

<!-- verify: manual, SRS-04:start:end, proof: ac-1.log-->
- [x] [SRS-04/AC-01] README and supporting docs publish the cloud Linux support matrix and remote operator workflow using canonical Port CLI and model terms. <!-- [SRS-04/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && /tmp/port-target/debug/port --help | rg -n "Cloud Linux|AWS|GCP|Azure|PVM" && rg -n "Cloud Linux|AWS|GCP|Azure|remote Linux|port doctor|port machine launch" /home/alex/workspace/spoke-sh/port/README.md /home/alex/workspace/spoke-sh/port/docs/operators.md /home/alex/workspace/spoke-sh/port/docs/cloud.md', proof: ac-1.log-->
<!-- verify: manual, SRS-05:start:end, proof: ac-2.log-->
- [x] [SRS-05/AC-01] The shipped docs and planning artifacts record the explicit research-backed decision to drop the PVM lane from the MVP. <!-- [SRS-05/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "PVM|protected VM|confidential VM|drop" /home/alex/workspace/spoke-sh/port/README.md /home/alex/workspace/spoke-sh/port/docs/cloud.md /home/alex/workspace/spoke-sh/port/.keel/epics/1vydg7000/voyages/1vyeq5000/SRS.md /home/alex/workspace/spoke-sh/port/.keel/epics/1vydg7000/voyages/1vyeq5000/SDD.md', proof: ac-2.log-->

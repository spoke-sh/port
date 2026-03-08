---
id: 1vz5nm000
title: Publish Hosted SDK And API Clients
type: feat
status: done
created_at: 2026-03-07T20:31:38
updated_at: 2026-03-08T06:46:05
scope: 1vz4Yn000/1vz5mg000
started_at: 2026-03-08T06:39:14
submitted_at: 2026-03-08T06:46:02
completed_at: 2026-03-08T06:46:05
---

# Publish Hosted SDK And API Clients

## Summary

Publish supported hosted SDK and API client surfaces once the hosted runtime and
service verbs stabilize.

## Acceptance Criteria

<!-- verify: manual, SRS-06:start:end, proof: ac-1.log, ac-2.log -->
- [x] [SRS-06/AC-01] Port publishes a supported SDK and API client surface for hosted machine, guest, and service operations that mirrors the canonical operator model. <!-- [SRS-06/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5nm000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-06/AC-02] README, docs, and examples show the intended SDK/API client entry points and call out any surfaces that remain planned rather than shipped. <!-- [SRS-06/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz5nm000/verify-ac-2.sh, proof: ac-2.log -->

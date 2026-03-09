---
id: 1vzWCK000
title: Publish Hosted Artifact Mobility Workflow
type: feat
status: backlog
created_at: 2026-03-09T01:42:44
updated_at: 2026-03-09T01:45:28
scope: 1vzW8e000/1vzW9Q000
---

# Publish Hosted Artifact Mobility Workflow

## Summary

Publish the first hosted artifact mobility workflow through README, artifact
docs, CLI help, and executable proof so operators can build, push, remove, and
pull a selected artifact variant end-to-end while understanding that OCI
support remains follow-on work.

## Acceptance Criteria

- [ ] [SRS-04/AC-01] Repo-local proof builds a selected artifact variant, pushes it to the hosted backend, removes the local output, then pulls the same variant back successfully through the canonical CLI.
- [ ] [SRS-05/AC-01] README, `docs/artifacts.md`, and relevant CLI help publish the hosted artifact workflow, control-plane store ownership, and auth expectations while explicitly stating that OCI remains follow-on work.
- [ ] [SRS-05/AC-02] The voyage closes with recorded board evidence and verification for the shipped hosted backend rather than leaving `hosted-api` as a modeled-only placeholder.

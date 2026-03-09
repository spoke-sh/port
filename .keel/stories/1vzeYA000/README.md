---
id: 1vzeYA000
title: Publish OCI Artifact Operator Workflow
type: feat
status: in-progress
created_at: 2026-03-09T10:37:50
updated_at: 2026-03-09T11:15:01
scope: 1vzW8e000/1vzeWr000
started_at: 2026-03-09T11:15:01
---

# Publish OCI Artifact Operator Workflow

## Summary

Publish the shipped OCI artifact workflow across the CLI, docs, examples, and
helper tasks so operators can discover and execute a local registry
build/push/remove/pull proof without leaving Port’s canonical artifact surface.

## Acceptance Criteria

<!-- verify: command, SRS-05:start, proof: ac-1.log -->
- [ ] [SRS-05/AC-01] README, `docs/artifacts.md`, CLI help, sample-config guidance, and helper tasks publish the executable local OCI registry workflow and its prerequisites. <!-- [SRS-05/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "oci-registry|oras|zot|demo-push-oci|demo-pull-oci" README.md docs/artifacts.md examples/port.toml justfile crates/port-cli/src/lib.rs', proof: ac-2.log -->
<!-- verify: command, SRS-05:end -->
<!-- verify: command, SRS-05:start, proof: ac-2.log -->
- [ ] [SRS-05/AC-02] Port records a repo-local proof that builds a variant, pushes it to a local OCI registry, removes the local artifact copy, and pulls it back without depending on a public registry, satisfying `SRS-NFR-02`. <!-- [SRS-05/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && just demo-push-oci && just demo-pull-oci', proof: ac-2.log -->
<!-- verify: command, SRS-05:end -->

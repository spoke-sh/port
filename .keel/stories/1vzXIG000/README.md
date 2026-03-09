---
id: 1vzXIG000
title: Publish Hosted Standard Cloud Workflow
type: feat
status: backlog
created_at: 2026-03-09T02:56:42
updated_at: 2026-03-09T02:56:09
scope: 1vzXFf000/1vzXFy000
---

# Publish Hosted Standard Cloud Workflow

## Summary

Publish the shipped hosted standard-lane cloud workflow through README, cloud
docs, hosted docs, and CLI help so operators can discover and execute the
`cloud-generic`, `cloud-aws`, and `cloud-gcp` demo lane without stale denial
guidance.

## Acceptance Criteria

<!-- verify: command, SRS-04:start, proof: ac-1.log -->
- [ ] [SRS-04/AC-01] README, `docs/cloud.md`, `docs/hosted.md`, and relevant CLI help publish the hosted standard cloud workflow for the sample `generic-linux`, `aws`, and `gcp` machines. <!-- [SRS-04/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-cli tests::help_includes_primary_surfaces -- --exact && rg -n \"cloud-generic|cloud-aws|cloud-gcp|hosted\" README.md docs/cloud.md docs/hosted.md crates/port-cli/src/lib.rs', proof: ac-1.log -->
<!-- verify: command, SRS-04:end, proof: ac-2.log -->
- [ ] [SRS-04/AC-02] The published workflow includes executable repo-local proof and removes stale guidance that tells operators to run Port directly on the provider host for these shipped demo lanes. <!-- [SRS-04/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-cli --test machine_commands cli_hosted_standard_cloud_launch_round_trip && rg -n \"run Port on the AWS Linux host itself|run Port on the GCP Linux host itself|run Port on that Linux host directly\" README.md docs/cloud.md docs/hosted.md crates/port-cli/src/lib.rs', proof: ac-2.log -->

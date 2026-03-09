---
id: 1vzXIG000
title: Publish Hosted Standard Cloud Workflow
type: feat
status: in-progress
created_at: 2026-03-09T02:56:42
updated_at: 2026-03-09T03:26:32
scope: 1vzXFf000/1vzXFy000
started_at: 2026-03-09T03:26:32
---

# Publish Hosted Standard Cloud Workflow

## Summary

Publish the shipped hosted standard-lane cloud workflow through README, cloud
docs, hosted docs, and CLI help so operators can discover and execute the
`cloud-generic`, `cloud-aws`, and `cloud-gcp` demo lane without stale denial
guidance.

## Acceptance Criteria

<!-- verify: command, SRS-04:start, proof: ac-1.log -->
- [x] [SRS-04/AC-01] README, `docs/cloud.md`, `docs/hosted.md`, and relevant CLI help publish the hosted standard cloud workflow for the sample `generic-linux`, `aws`, and `gcp` machines. <!-- [SRS-04/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && bash scripts/verify-hosted-standard-docs.sh', proof: ac-1.log -->
<!-- verify: command, SRS-04:end, proof: ac-2.log -->
- [x] [SRS-04/AC-02] The published workflow includes executable repo-local proof and removes stale guidance that tells operators to run Port directly on the provider host for these shipped demo lanes. <!-- [SRS-04/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-cli --test machine_commands cli_hosted_standard_cloud_launch_round_trip && cargo test -q -p port-cli --test machine_commands cli_hosted_standard_status_and_stop_round_trip && ! grep -nE \"run Port on the AWS Linux host itself|run Port on the GCP Linux host itself|run Port on that Linux host directly\" README.md docs/cloud.md docs/hosted.md crates/port-cli/src/lib.rs', proof: ac-2.log -->

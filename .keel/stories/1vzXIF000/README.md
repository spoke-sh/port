---
id: 1vzXIF000
title: Route Standard Cloud Launch Through Hosted Runtime
type: feat
status: in-progress
created_at: 2026-03-09T02:56:41
updated_at: 2026-03-09T03:06:46
scope: 1vzXFf000/1vzXFy000
started_at: 2026-03-09T03:06:46
---

# Route Standard Cloud Launch Through Hosted Runtime

## Summary

Route the sample `generic-linux`, `aws`, and `gcp` standard Firecracker lanes
through the live hosted control-plane and node-agent runtime so canonical
`machine launch|status|stop` operate on registered remote nodes instead of
failing fast with provider guidance.

## Acceptance Criteria

<!-- verify: command, SRS-02:start, proof: ac-1.log -->
- [ ] [SRS-02/AC-01] `port machine launch --machine cloud-generic`, `cloud-aws`, and `cloud-gcp` route through the hosted control plane and selected node agent, and the selected node owns the runtime root for the launched machine. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_standard_launch && cargo test -q -p port-cli --test machine_commands cli_hosted_standard_cloud_launch_round_trip', proof: ac-2.log -->
<!-- verify: command, SRS-02:end, proof: ac-3.log -->
- [ ] [SRS-02/AC-02] Hosted standard-lane launch failures surface machine, host, provider, control plane, and selected-node detail without falling back to the local launch path, satisfying `SRS-NFR-01` and `SRS-NFR-02`. <!-- [SRS-02/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_standard_launch', proof: ac-2.log -->
<!-- verify: command, SRS-03:start:end, proof: ac-3.log -->
- [ ] [SRS-03/AC-01] `port machine status` and `port machine stop` work for hosted standard-lane machines using stored placement, and their output includes provider plus hosted-node routing detail. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime hosted_standard_status_stop && cargo test -q -p port-cli --test machine_commands cli_hosted_standard_status_and_stop_round_trip', proof: ac-3.log -->

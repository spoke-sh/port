---
id: 1vydit000
title: Build Artifact Pipelines And Docs
type: feat
status: done
created_at: 2026-03-06T14:32:43
updated_at: 2026-03-06T15:38:58
scope: 1vydg7000/1vydgL000
started_at: 2026-03-06T15:23:45
submitted_at: 2026-03-06T15:38:54
completed_at: 2026-03-06T15:38:58
---

# Build Artifact Pipelines And Docs

## Summary

Build the kernel and guest-image pipelines used by the local MVP path, validate
their outputs, and document the artifact contracts and operator-facing build
workflow.

## Acceptance Criteria

<!-- verify: manual, SRS-05:start:end, proof: ac-1.log-->
- [x] [SRS-05/AC-01] A reproducible kernel build pipeline exists in-repo and emits a documented kernel artifact for Firecracker. <!-- [SRS-05/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && nix develop -c cargo run -p port-cli -- --config /home/alex/workspace/spoke-sh/port/examples/port.toml artifacts build --artifact demo-kernel && rg -n "demo-kernel|Kernel Artifact" /home/alex/workspace/spoke-sh/port/README.md /home/alex/workspace/spoke-sh/port/docs/artifacts.md', proof: ac-2.log-->
- [x] [SRS-05/AC-02] Validation commands or checks exist for kernel and guest-image artifacts and are recorded as evidence. <!-- [SRS-05/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && nix develop -c cargo run -p port-cli -- --config /home/alex/workspace/spoke-sh/port/examples/port.toml artifacts validate --artifact demo-kernel && nix develop -c cargo run -p port-cli -- --config /home/alex/workspace/spoke-sh/port/examples/port.toml artifacts validate --artifact demo-guest', proof: ac-3.log-->
<!-- verify: manual, SRS-06:start:end, proof: ac-4.log-->
- [x] [SRS-06/AC-01] A reproducible guest-image build pipeline exists in-repo and emits a documented guest-image artifact with the Port guest agent. <!-- [SRS-06/AC-01] verify: bash /tmp/port-proof-artifacts/verify-built-guest-launch.sh, proof: ac-5.log-->

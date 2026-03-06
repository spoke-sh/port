---
id: 1vydit000
title: Build Artifact Pipelines And Docs
type: feat
status: in-progress
created_at: 2026-03-06T14:32:43
updated_at: 2026-03-06T15:23:45
scope: 1vydg7000/1vydgL000
started_at: 2026-03-06T15:23:45
---

# Build Artifact Pipelines And Docs

## Summary

Build the kernel and guest-image pipelines used by the local MVP path, validate
their outputs, and document the artifact contracts and operator-facing build
workflow.

## Acceptance Criteria

<!-- verify: manual, SRS-05:start:end, proof: ac-1.log-->
- [x] [SRS-05/AC-01] A reproducible kernel build pipeline exists in-repo and emits a documented kernel artifact for Firecracker. <!-- [SRS-05/AC-01] verify: bash -lc 'nix develop -c cargo run -p port-cli -- --config examples/port.toml artifacts build --artifact demo-kernel && rg -n "demo-kernel|Kernel Artifact" README.md docs/artifacts.md', proof: ac-2.log-->
- [x] [SRS-05/AC-02] Validation commands or checks exist for kernel and guest-image artifacts and are recorded as evidence. <!-- [SRS-05/AC-02] verify: bash -lc 'nix develop -c cargo run -p port-cli -- --config examples/port.toml artifacts validate --artifact demo-kernel && nix develop -c cargo run -p port-cli -- --config examples/port.toml artifacts validate --artifact demo-guest', proof: ac-3.log-->
<!-- verify: manual, SRS-06:start:end, proof: ac-4.log-->
- [x] [SRS-06/AC-01] A reproducible guest-image build pipeline exists in-repo and emits a documented guest-image artifact with the Port guest agent. <!-- [SRS-06/AC-01] verify: bash -lc 'nix develop -c cargo run -p port-cli -- --config examples/port.toml artifacts build --artifact demo-guest && nix develop -c cargo run -p port-cli -- --config examples/port.toml artifacts validate --artifact demo-guest && runtime_root=/tmp/port-artifact-proof-$(date +%s) && nix develop -c cargo run -p port-cli -- --config examples/port.toml machine launch --machine demo --runtime-root "$runtime_root" --boot-wait-secs 5 && rg -q "Mounted root \\(ext4 filesystem\\)" "$runtime_root/demo/console.stdout.log" && kill "$(cat "$runtime_root/demo/firecracker.pid")" && rg -n "demo-guest|Guest Image Artifact|port-guest-agent" README.md docs/artifacts.md', proof: ac-5.log-->

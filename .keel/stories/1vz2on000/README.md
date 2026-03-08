---
id: 1vz2on000
title: Define Artifact Mobility Commands And Contracts
type: feat
status: done
created_at: 2026-03-07T17:20:29
updated_at: 2026-03-07T18:11:30
scope: 1vz2eV000/1vz2ky000
started_at: 2026-03-07T17:56:43
submitted_at: 2026-03-07T18:11:24
completed_at: 2026-03-07T18:11:30
---

# Define Artifact Mobility Commands And Contracts

## Summary

Turn artifacts into a real product surface for local and remote use by defining
canonical references, compatibility metadata, and discoverable build, push, and
pull semantics.

## Acceptance Criteria

<!-- verify: manual, SRS-05:start:end, proof: ac-1.log-->
- [x] [SRS-05/AC-01] Port defines canonical artifact-reference and compatibility concepts covering local outputs, remote references, architecture, backend, and protection-mode variants. <!-- [SRS-05/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && env CARGO_TARGET_DIR=/tmp/port-target cargo test -q -p port-model', proof: ac-1.log-->
<!-- verify: manual, SRS-06:start:end, proof: ac-2.log-->
- [x] [SRS-06/AC-01] The CLI surface and help text expose discoverable artifact mobility commands or reserved subcommands for build, push, and pull workflows. <!-- [SRS-06/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo run -q -p port-cli -- artifacts --help && cargo run -q -p port-cli -- artifacts push --help', proof: ac-2.log-->
<!-- verify: manual, SRS-06:start:end, proof: ac-3.log-->
- [x] [SRS-06/AC-02] Port publishes operator-facing documentation for local build, remote pull, and compatibility-selection flows using the new artifact vocabulary. <!-- [SRS-06/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "artifacts push|artifacts pull|Artifact Contracts|file-backed store|--architecture|reference|variants|cache" README.md docs/operators.md docs/artifacts.md docs/cloud.md', proof: ac-3.log-->
<!-- verify: manual, SRS-05:start:end, proof: ac-4.log-->
- [x] [SRS-05/AC-04] The story defines concrete verification hooks for artifact mobility behavior through tests, docs review, and CLI-level evidence. <!-- [SRS-05/AC-04] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && env CARGO_TARGET_DIR=/tmp/port-target cargo test -q -p port-cli && rm -rf artifact-store/demo-fs .port/cache artifacts/kernel/demo && mkdir -p artifacts/kernel/demo/x86_64/firecracker/standard && printf demo-kernel-proof > artifacts/kernel/demo/x86_64/firecracker/standard/vmlinux && cargo run -q -p port-cli -- --config examples/port.toml artifacts push --artifact demo-kernel --architecture x86-64 && rm -f artifacts/kernel/demo/x86_64/firecracker/standard/vmlinux && cargo run -q -p port-cli -- --config examples/port.toml artifacts pull --artifact demo-kernel --architecture x86-64 && rm -rf artifact-store/demo-fs .port/cache artifacts/kernel/demo', proof: ac-4.log-->

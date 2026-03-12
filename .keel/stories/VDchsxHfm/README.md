---
id: VDchsxHfm
title: Add Install Proof For Packaged Port
type: feat
status: backlog
created_at: 2026-03-11T21:16:30
updated_at: 2026-03-11T21:19:02
scope: VDcT0vaPb/VDchK6xzs
index: 3
---

# Add Install Proof For Packaged Port

## Summary

Prove that a packaged Port artifact can be extracted or installed into a clean
prefix and used through the canonical binary path without relying on repo-local
Cargo commands or external release infrastructure.

## Acceptance Criteria

<!-- verify: command, SRS-03:start:end, proof: ac-1.log -->
- [ ] [SRS-03/AC-01] The install proof extracts or installs the packaged artifact and runs the packaged `port` binary successfully for `--version` and `doctor` without falling back to `cargo run -p port-cli`. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && just package-proof x86_64-unknown-linux-gnu', proof: ac-1.log -->
<!-- verify: command, SRS-NFR-03:start:end, proof: ac-2.log -->
- [ ] [SRS-NFR-03/AC-02] The package proof remains repo-local and can be recorded without external release credentials or hosted publication infrastructure. <!-- [SRS-NFR-03/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && vhs artifacts/package-proof.tape', proof: ac-2.log -->

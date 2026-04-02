---
# system-managed
id: VFdhWcOqz
status: done
created_at: 2026-04-02T06:19:06
updated_at: 2026-04-02T06:33:27
# authored
title: Add Cargo-Dist Release Flow And Port Upgrade Command
type: feat
operator-signal:
scope: VFdgQWhbn/VFdgVAzQc
index: 1
started_at: 2026-04-02T06:19:54
completed_at: 2026-04-02T06:33:27
---

# Add Cargo-Dist Release Flow And Port Upgrade Command

## Summary

Add the first cargo-dist release contract for Port, wire the release workflow to
the supported target matrix, and expose a `port upgrade` command that installs
either the latest release or a requested git revision through the new installer
path without breaking packaged asset lookup.

## Acceptance Criteria

- [x] [SRS-01/AC-01] Port defines cargo-dist workspace metadata that mirrors the Keel and Sift release model while keeping Port's supported target matrix explicit. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -p port --test package_workflow --test package_proof --test upgrade_commands', SRS-01:start:end -->
- [x] [SRS-02/AC-02] Port defines a tag-driven release workflow that plans on pull requests and publishes artifacts on version tags. <!-- [SRS-02/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && ~/.cargo/bin/dist plan', SRS-02:start:end, SRS-NFR-02:end -->
- [x] [SRS-03/AC-03] `port upgrade` installs the latest released Port binary through the published installer contract when no revision is specified. <!-- [SRS-03/AC-03] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -p port --test upgrade_commands upgrade_runs_release_installer_for_latest_version', SRS-03:start:end, SRS-NFR-01:end -->
- [x] [SRS-04/AC-04] `port upgrade --tag <tag>` and `port upgrade --sha <sha>` reuse `~/.cache/port`, build with a supported local Rust toolchain, and install the requested revision. <!-- [SRS-04/AC-04] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -p port --test upgrade_commands upgrade_builds_and_installs', SRS-04:start:end, SRS-NFR-01:end -->
- [x] [SRS-05/AC-05] Installed Port binaries resolve bundled runtime assets from both legacy packaged layouts and cargo-dist-installed layouts. <!-- [SRS-05/AC-05] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -p port-runtime artifact_scripts_resolve_from_packaged', SRS-05:start:end, SRS-NFR-01:end -->

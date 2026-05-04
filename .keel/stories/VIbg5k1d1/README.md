---
# system-managed
id: VIbg5k1d1
status: done
created_at: 2026-05-03T17:27:30
updated_at: 2026-05-03T17:37:01
# authored
title: Use K3s 1.35.4 From Nixpkgs
type: chore
operator-signal:
started_at: 2026-05-03T17:27:32
completed_at: 2026-05-03T17:37:01
---

# Use K3s 1.35.4 From Nixpkgs

## Summary

Use the merged NixOS/nixpkgs K3s update from PR 515339 by moving Port's
primary nixpkgs input to that revision, so the packaged runtime and
development shell resolve `k3s` to `1.35.4+k3s1` without a dedicated K3s-only
nixpkgs input.

## Acceptance Criteria

- [x] [SRS-01/AC-01] The flake routes Port's Linux K3s runtime and dev-shell dependency through the primary nixpkgs input, and that input resolves `pkgs.k3s.version` to `1.35.4+k3s1` without `nixpkgs-k3s`. <!-- verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && ! rg -n "nixpkgs-k3s|k3sPkg" flake.nix flake.lock && nix --quiet develop --command k3s --version | rg "v1\\.35\\.4\\+k3s1"', SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-02] Hosted K3s source fixtures and tests now use `v1.35.4+k3s1` instead of `v1.35.2+k3s1`. <!-- verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && ! rg -n "v1\\.35\\.2\\+k3s1|1\\.35\\.2\\+k3s1" crates scripts', SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-03/AC-03] Focused Rust and script checks pass for the touched hosted K3s surfaces. <!-- verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo fmt --all -- --check && cargo test -q -p port-model hosted_k3s && cargo test -q -p port-runtime hosted_k3s_cluster_access_contract && cargo test -q -p port --test machine_commands cli_cluster_show_and_lifecycle_surface_hosted_k3s_microvms && bash -n scripts/render-hosted-k3s-proof.sh scripts/render-hosted-k3s-ha-failover-proof.sh', SRS-03:start:end, proof: ac-3.log -->

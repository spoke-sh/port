---
# system-managed
id: VGafyVDGA
status: done
created_at: 2026-04-12T08:23:00
updated_at: 2026-04-12T09:41:04
# authored
title: Capture Hosted AWS PVM Failover Proof For The Stable Endpoint
type: feat
operator-signal:
scope: VGYFpfmpi/VGafx2vn4
index: 2
started_at: 2026-04-12T09:22:00
submitted_at: 2026-04-12T09:40:37
completed_at: 2026-04-12T09:41:04
---

# Capture Hosted AWS PVM Failover Proof For The Stable Endpoint

## Summary

Capture one human-reviewable failover proof for the hosted AWS PVM HA endpoint
so Port's first real-HA claim is backed by executable evidence rather than a
documentation promise.

## Acceptance Criteria

- [x] [SRS-03/AC-01] The canonical proof workflow shows the stable endpoint working before and after one supported control-plane host-loss or guest-replacement scenario on hosted AWS PVM. <!-- verify: command, SRS-03:start:end -->
- [x] [SRS-NFR-01/AC-02] The failover proof is stored as a human-reviewable Port proof artifact rather than as chat-only notes. <!-- verify: manual, SRS-NFR-01:start:end -->
- [x] [SRS-NFR-02/AC-03] The proof or its paired negative-path evidence makes missing failover prerequisites explicit instead of implying stability that Port cannot yet provide. <!-- verify: command, SRS-NFR-02:start:end -->

## Proof

- AC-01: `scripts/render-hosted-k3s-ha-failover-proof.sh` generated `EVIDENCE/hosted-k3s-ha-failover-workflow.cast` and `EVIDENCE/ac-1.gif`, with `EVIDENCE/ac-1.log` capturing the render run. The artifact shows `port cluster status` and `port cluster kubeconfig` before and after a primary hosted control-plane guest replacement while the configured `api_endpoint` remains `https://demo-k3s.internal:6443`.
- AC-02: The cast and gif are committed story artifacts under `EVIDENCE/`, so the proof remains human-reviewable from the repository surface rather than chat history.
- AC-03: `EVIDENCE/ac-2.log` captures the paired negative path from `port cluster show --cluster demo` for a single-control-plane hosted contract, which explicitly states that real HA still depends on at least three control-plane microVMs, a stable HTTPS api endpoint, and distinct execution hosts behind that endpoint.
- Verification: `EVIDENCE/ac-3.log` records `cargo test -q -p port --test machine_commands cli_cluster_status_surfaces_hosted_real_ha_truth -- --nocapture`, and `EVIDENCE/ac-4.log` records `cargo fmt --check`.

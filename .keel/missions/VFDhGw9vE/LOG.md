# Ship Simple Port Cluster Bootstrap Surface - Decision Log

<!-- Append entries below. Each entry is an H2 with ISO timestamp. -->
<!-- Use `keel mission digest` to compress older entries when this file grows large. -->

## 2026-03-28T19:40:00Z

- Created this mission after the downstream `infra` repo proved that Port's
  current hosted-K3s contract is still too low-level for a clean operator
  experience.
- The concrete gap is that infra had to manage control-plane daemons, node
  agents, machine launches, guest install commands, join-token capture, API
  forwarding, and kubeconfig rewriting itself.
- This mission reframes the next Port slice around a simple cluster-first
  operator contract inspired by Slicer's K3s workflow.

## 2026-03-28T19:50:26

Refined epic VFDhlRjOf around a cluster-first local K3s contract, planned voyage VFDk8fdnG for the first single-node local slice, and decomposed four execution stories covering cluster CLI/config, offline bootstrap inputs, lifecycle plus kubeconfig health surfaces, and docs or proof handoff. The recommendation maps to this mission directly; multi-node or AWS expansion remains explicit follow-on scope.

## 2026-03-28T21:03:04

Completed VFDk8fqnH: added the cluster contract surface, sample config, and fail-fast single-node local validation with cluster list/show proofs and full quality coverage.

## 2026-03-28T21:35:31

Accepted VFDk8gGoC after adding the offline local cluster bootstrap contract, repo-owned demo K3s kit, `port cluster stage`, and proof-backed repo-local verification without guest-side live fetches.

## 2026-03-28T22:04:21

Accepted VFDk8gRoD. Port now owns local cluster lifecycle status, kubeconfig handoff, and deterministic API forward cleanup for the first single-node K3s slice. Direct operator proof and just quality both passed.

## 2026-03-29T08:36:27

Completed VFDk8ggoV, closed voyage VFDk8fdnG, and finalized epic VFDhlRjOf with a local-cluster proof-backed mission surface.

## 2026-03-29T08:36:27

Mission achieved by local system user 'alex'

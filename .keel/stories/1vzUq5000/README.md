---
id: 1vzUq5000
title: Define Durable Hosted Registry Contract
type: feat
status: backlog
created_at: 2026-03-09T00:15:41
updated_at: 2026-03-09T00:20:39
scope: 1vzUnI000/1vzUoK000
---

# Define Durable Hosted Registry Contract

## Summary

Define the shared durable hosted registry contract in Port’s model and hosted
protocol so the control plane can represent persisted node registration,
freshness, and imported inventory provenance through one canonical identity
namespace.

## Acceptance Criteria

<!-- verify: command, SRS-06:start, proof: ac-1.log -->
- [ ] [SRS-06/AC-01] Shared model and hosted protocol types represent persisted hosted node registration records with node identity, endpoint, registration time, last-seen time, and freshness state. <!-- [SRS-06/AC-01] verify: cargo test -q -p port-model durable_hosted_registry && cargo test -q -p port-hosted-protocol durable_hosted_registry, proof: ac-1.log -->
<!-- verify: command, SRS-06:end, proof: ac-2.log -->
- [ ] [SRS-06/AC-02] Shared contracts represent imported inventory records and provenance metadata that merge onto canonical configured node names without introducing a second fleet namespace. <!-- [SRS-06/AC-02] verify: cargo test -q -p port-model durable_hosted_registry && cargo test -q -p port-hosted-protocol durable_hosted_registry, proof: ac-2.log -->
<!-- verify: command, SRS-06:start, proof: ac-3.log -->
- [ ] [SRS-06/AC-03] Contract validation and serialization errors include explicit durable-registry or import context together with affected-node detail, satisfying `SRS-NFR-02`. <!-- [SRS-06/AC-03] verify: cargo test -q -p port-model durable_hosted_registry && cargo test -q -p port-hosted-protocol durable_hosted_registry, proof: ac-3.log -->
<!-- verify: command, SRS-06:end, proof: ac-3.log -->

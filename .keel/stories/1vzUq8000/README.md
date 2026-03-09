---
id: 1vzUq8000
title: Surface Durable Hosted Fleet State
type: feat
status: backlog
created_at: 2026-03-09T00:16:44
updated_at: 2026-03-09T00:20:39
scope: 1vzUnI000/1vzUoK000
---

# Surface Durable Hosted Fleet State

## Summary

Surface persisted hosted registration, freshness, and imported inventory
provenance through canonical machine or fleet inspection output so operators
can understand which nodes are configured, imported, live, stale, or ineligible
without reading runtime files directly.

## Acceptance Criteria

<!-- verify: command, SRS-04:start, proof: ac-1.log -->
- [ ] [SRS-04/AC-01] Canonical hosted inspection output reports configured, imported, registered, freshness, and routing-eligibility state for each node instead of collapsing the fleet into generic hosted status. <!-- [SRS-04/AC-01] verify: cargo test -q -p port-runtime hosted_fleet_state, proof: ac-1.log -->
<!-- verify: command, SRS-04:end, proof: ac-2.log -->
- [ ] [SRS-04/AC-02] Stale, imported-only, and missing-registration nodes remain visible in hosted inspection output with explicit state detail. <!-- [SRS-04/AC-02] verify: cargo test -q -p port-runtime hosted_fleet_state, proof: ac-2.log -->
<!-- verify: command, SRS-04:start, proof: ac-3.log -->
- [ ] [SRS-04/AC-03] Hosted inspection failures include control-plane context and affected-node detail when fleet state cannot be loaded or merged, satisfying `SRS-NFR-02`. <!-- [SRS-04/AC-03] verify: cargo test -q -p port-runtime hosted_fleet_state, proof: ac-3.log -->
<!-- verify: command, SRS-04:end, proof: ac-3.log -->

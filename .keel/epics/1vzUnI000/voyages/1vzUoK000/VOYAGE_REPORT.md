# VOYAGE REPORT: Persistent Registration And Inventory Sync

## Voyage Metadata
- **ID:** 1vzUoK000
- **Epic:** 1vzUnI000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 5/5 stories complete

## Implementation Narrative
### Define Durable Hosted Registry Contract
- **ID:** 1vzUq5000
- **Status:** done

#### Summary
Define the shared durable hosted registry contract in Port’s model and hosted
protocol so the control plane can represent persisted node registration,
freshness, and imported inventory provenance through one canonical identity
namespace.

#### Acceptance Criteria
- [x] [SRS-06/AC-01] Shared model and hosted protocol types represent persisted hosted node registration records with node identity, endpoint, registration time, last-seen time, and freshness state. <!-- [SRS-06/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzUq5000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-06/AC-02] Shared contracts represent imported inventory records and provenance metadata that merge onto canonical configured node names without introducing a second fleet namespace. <!-- [SRS-06/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzUq5000/verify-ac-2.sh, proof: ac-2.log -->
- [x] [SRS-06/AC-03] Contract validation and serialization errors include explicit durable-registry or import context together with affected-node detail, satisfying `SRS-NFR-02`. <!-- [SRS-06/AC-03] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzUq5000/verify-ac-3.sh, proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzUq5000/EVIDENCE/ac-1.log)
- [ac-3.log](../../../../stories/1vzUq5000/EVIDENCE/ac-3.log)
- [ac-2.log](../../../../stories/1vzUq5000/EVIDENCE/ac-2.log)

### Persist Hosted Registration And Freshness
- **ID:** 1vzUq6000
- **Status:** done

#### Summary
Persist hosted node registration and heartbeat freshness under the control-plane
runtime root and refresh that state through the existing hosted node-agent
transport so the fleet view survives restart and stale nodes become explicitly
ineligible.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] The hosted control plane stores and reloads durable node registration records so restart reconstructs the fleet view from runtime-owned state instead of losing node presence. <!-- [SRS-01/AC-01] verify: cargo test -q -p port-runtime hosted_registry_persistence, proof: ac-1.log -->
- [x] [SRS-02/AC-02] `port node-agent serve` refreshes registration and heartbeat freshness through the existing hosted auth and transport contract without a second token or registration path. <!-- [SRS-02/AC-02] verify: cargo test -q -p port-runtime hosted_registry_persistence, proof: ac-2.log -->
- [x] [SRS-01/AC-03] Restart recovery and freshness expiry behave deterministically for the same stored registry state and current time inputs, satisfying `SRS-NFR-01`. <!-- [SRS-01/AC-03] verify: cargo test -q -p port-runtime hosted_registry_persistence, proof: ac-3.log -->
- [x] [SRS-02/AC-04] Stale-node or durable-registry failures include explicit control-plane path context and affected-node detail, satisfying `SRS-NFR-02`. <!-- [SRS-02/AC-04] verify: cargo test -q -p port-runtime hosted_registry_persistence, proof: ac-4.log -->

#### Implementation Insights
- **1vzVJp000: Isolate Default Hosted Test State From Demo Runtime Files**
  - Insight: Persisted hosted registry and placement files under `.port/hosted/demo` can override static test bindings because registered nodes are resolved before static bindings. Isolated reruns then fail differently from full suites depending on leftover on-disk state.
  - Suggested Action: Clear default hosted runtime state in test helpers before starting a static-bound control plane, or use unique control-plane names per test when persisted state is part of the scenario.
  - Applies To: `crates/port-runtime/src/hosted_control_plane.rs`, hosted control-plane tests
  - Category: testing


#### Verified Evidence
- [ac-4.log](../../../../stories/1vzUq6000/EVIDENCE/ac-4.log)
- [ac-1.log](../../../../stories/1vzUq6000/EVIDENCE/ac-1.log)
- [ac-3.log](../../../../stories/1vzUq6000/EVIDENCE/ac-3.log)
- [ac-2.log](../../../../stories/1vzUq6000/EVIDENCE/ac-2.log)

### Materialize Imported Fleet Inventory
- **ID:** 1vzUq7000
- **Status:** done

#### Summary
Materialize an imported fleet inventory contract into the hosted control-plane
state so Port can merge externally supplied node membership and provenance with
configured nodes before routing or inspection occurs.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] Port accepts and persists an imported inventory contract that records node membership, provider, provenance, and capability summary under the hosted control-plane state. <!-- [SRS-03/AC-01] verify: cargo test -q -p port-runtime hosted_imported_inventory, proof: ac-2.log -->
- [x] [SRS-03/AC-02] Imported inventory merges onto canonical configured node identities and reports unknown-node or conflicting imports explicitly instead of silently inventing new runtime-only nodes. <!-- [SRS-03/AC-02] verify: cargo test -q -p port-runtime hosted_imported_inventory, proof: ac-2.log -->
- [x] [SRS-03/AC-03] Import mismatch and persistence failures include durable import path context and affected-node detail, satisfying `SRS-NFR-02`. <!-- [SRS-03/AC-03] verify: cargo test -q -p port-runtime hosted_imported_inventory, proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzUq7000/EVIDENCE/ac-1.log)
- [ac-3.log](../../../../stories/1vzUq7000/EVIDENCE/ac-3.log)
- [ac-2.log](../../../../stories/1vzUq7000/EVIDENCE/ac-2.log)

### Surface Durable Hosted Fleet State
- **ID:** 1vzUq8000
- **Status:** done

#### Summary
Surface persisted hosted registration, freshness, and imported inventory
provenance through canonical machine or fleet inspection output so operators
can understand which nodes are configured, imported, live, stale, or ineligible
without reading runtime files directly.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] Canonical hosted inspection output reports configured, imported, registered, freshness, and routing-eligibility state for each node instead of collapsing the fleet into generic hosted status. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzUq8000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-04/AC-02] Stale, imported-only, and missing-registration nodes remain visible in hosted inspection output with explicit state detail. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzUq8000/verify-ac-2.sh, proof: ac-2.log -->
- [x] [SRS-04/AC-03] Hosted inspection failures include control-plane context and affected-node detail when fleet state cannot be loaded or merged, satisfying `SRS-NFR-02`. <!-- [SRS-04/AC-03] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzUq8000/verify-ac-3.sh, proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzUq8000/EVIDENCE/ac-1.log)
- [ac-3.log](../../../../stories/1vzUq8000/EVIDENCE/ac-3.log)
- [ac-2.log](../../../../stories/1vzUq8000/EVIDENCE/ac-2.log)

### Publish Durable Hosted Fleet Workflow
- **ID:** 1vzUq9000
- **Status:** done

#### Summary
Publish the durable hosted fleet workflow through canonical CLI help, README,
hosted docs, and proof so operators can discover registration persistence,
heartbeat freshness, imported inventory, and the limits that remain after this
voyage.

#### Acceptance Criteria
- [x] [SRS-05/AC-01] CLI help, README, and hosted docs publish the durable registration, heartbeat freshness, and imported inventory workflow through canonical `port machine`, `port control-plane`, and `port node-agent` surfaces. <!-- [SRS-05/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzUq9000/verify-ac-1.sh, proof: ac-2.log -->
- [x] [SRS-05/AC-02] Repo-local proof covers restart recovery or imported-inventory inspection so operators can learn the workflow from executable evidence, not prose only. <!-- [SRS-05/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzUq9000/verify-ac-2.sh, proof: ac-2.log -->
- [x] [SRS-05/AC-03] The voyage closes with board evidence and verification on the implemented stories rather than leaving a second hosted planning-only backlog, satisfying `SRS-NFR-03`. <!-- [SRS-05/AC-03] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzUq9000/verify-ac-3.sh, proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzUq9000/EVIDENCE/ac-1.log)
- [ac-3.log](../../../../stories/1vzUq9000/EVIDENCE/ac-3.log)
- [ac-2.log](../../../../stories/1vzUq9000/EVIDENCE/ac-2.log)



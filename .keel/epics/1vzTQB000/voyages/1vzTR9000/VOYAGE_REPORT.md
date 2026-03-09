# VOYAGE REPORT: Registered Nodes And Machine Launch Placement

## Voyage Metadata
- **ID:** 1vzTR9000
- **Epic:** 1vzTQB000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 5/5 stories complete

## Implementation Narrative
### Publish Registered Hosted Machine Workflow
- **ID:** 1vzTSY000
- **Status:** done

#### Summary
Publish the repository-local registered-node hosted machine workflow so
operators can discover how nodes register and how hosted machine placement now
works through the canonical machine surface.

#### Acceptance Criteria
- [x] [SRS-05/AC-01] CLI help, README, hosted docs, and proof publish the registered-node hosted machine workflow through canonical `port machine` and `port node-agent serve` surfaces. <!-- [SRS-05/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzTSY000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-05/AC-02] Operator messaging makes explicit the remaining limits after this slice, including no autoscaling, broader fleet policy, or external inventory yet. <!-- [SRS-05/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzTSY000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzTSY000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzTSY000/EVIDENCE/ac-2.log)

### Define Registered Node Contract And State
- **ID:** 1vzTT1000
- **Status:** done

#### Summary
Define the shared registered-node contract and control-plane-owned registration
state so hosted machine placement can reason about live nodes without transient
`--node-binding` startup flags.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Shared model, hosted protocol, and runtime state define registered-node identity, endpoint, token, freshness, and placement-facing fields. <!-- [SRS-01/AC-01] verify: cargo test, proof: ac-1.log -->
- [x] [SRS-01/AC-02] Validation or diagnostics surface missing or invalid registered-node inputs with explicit detail. <!-- [SRS-01/AC-02] verify: cargo test, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzTT1000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzTT1000/EVIDENCE/ac-2.log)

### Implement Node Agent Registration Refresh
- **ID:** 1vzTTI000
- **Status:** done

#### Summary
Let `port node-agent serve` register one configured node against the hosted
control plane and refresh that registration while the node agent remains live.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] A running node agent registers its node against the hosted control plane and refreshes freshness state while it is serving. <!-- [SRS-02/AC-01] verify: cargo test -q -p port-runtime node_agent_registers_and_refreshes_against_control_plane, proof: ac-2.log -->
- [x] [SRS-02/AC-02] Registration failures such as unreachable control planes, auth mismatches, or stale registration are surfaced explicitly in hosted runtime proof output. <!-- [SRS-02/AC-02] verify: cargo test -q -p port-runtime node_agent_surfaces_explicit_registration_failures, proof: ac-2.log -->

#### Implementation Insights
- **1vzU8M000: Hosted node-agent tests must bootstrap the control plane first**
  - Insight: The node agent now fails before listening unless its configured control-plane endpoint is already reachable and the control-plane auth env var is present. Older hosted fixtures that started node agents before the control plane or omitted the token now fail for the right reason.
  - Suggested Action: In hosted fixtures, start the control plane first, set the control-plane token env var for the node agent, and isolate or clean `.port/hosted/<control-plane>` registration state between tests.
  - Applies To: `crates/port-cli/tests/*.rs`, hosted runtime integration helpers, future hosted proof scripts
  - Category: testing


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzTTI000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzTTI000/EVIDENCE/ac-2.log)

### Route Hosted Machine Launch Through Registered Nodes
- **ID:** 1vzTTJ000
- **Status:** done

#### Summary
Route canonical hosted `port machine launch` through registered nodes so the
control plane chooses one eligible live node and records the placement it used.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] Hosted `port machine launch` selects one eligible registered node and executes the existing node-owned launch path through that node. <!-- [SRS-03/AC-01] verify: cargo test -q -p port-cli --test machine_commands cli_machine_launch_routes_hosted_pvm_through_live_control_plane, proof: ac-1.log -->
- [x] [SRS-03/AC-02] Placement remains deterministic for the same registered-node input and rejects stale or ineligible nodes with explicit detail. <!-- [SRS-03/AC-02] verify: cargo test -q -p port-runtime control_plane_launch_, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzTTJ000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzTTJ000/EVIDENCE/ac-2.log)

### Surface Registered Placement Through Machine Commands
- **ID:** 1vzTTK000
- **Status:** done

#### Summary
Surface registered-node identity, placement detail, and stale-registration
failures through canonical hosted `port machine` output instead of a separate
fleet-only surface.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] Hosted `port machine list|status|monitor|stop` surface selected registered-node identity, freshness or registration state, and placement detail through canonical machine output. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzTTK000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-04/AC-02] Missing or stale registered-node state remains operator-visible through machine output instead of collapsing into generic hosted transport failures. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzTTK000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzTTK000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzTTK000/EVIDENCE/ac-2.log)



# VOYAGE REPORT: Implement Hosted Control Plane Demo Lane

## Voyage Metadata
- **ID:** 1vzETX000
- **Epic:** 1vzETR000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 5/5 stories complete

## Implementation Narrative
### Publish Hosted Demo Workflow And Evidence
- **ID:** 1vzEVX000
- **Status:** done

#### Summary
Publish the runnable hosted demo workflow, examples, and board evidence once the
control-plane and node-agent transport is live.

#### Acceptance Criteria
- [x] [SRS-05/AC-01] README, hosted docs, and CLI help show how to start the control plane and node agent, then run canonical hosted machine and guest commands end-to-end. <!-- [SRS-05/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzEVX000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-05/AC-02] Repository-local evidence proves the hosted demo workflow is reproducible and clearly calls out any remaining transport limits. <!-- [SRS-05/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzEVX000/verify-ac-2.sh, proof: ac-2.log -->

#### Implementation Insights
- **1w03mg000: Prefer Reusable Demo Scripts Over Test-Only Proof For Operator Workflows**
  - Insight: A small repo-local demo script is higher-signal than a test name for operator-facing evidence because it can be linked from docs, called by verification scripts, and run directly by humans without understanding the test harness.
  - Suggested Action: When a story is primarily about workflow discoverability or reproducibility, publish a reusable demo script and have the `keel` verification script call it instead of recording only crate-test commands.
  - Applies To: `scripts/*.sh`, `.keel/stories/*/verify-ac-*.sh`, operator workflow docs
  - Category: process


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzEVX000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzEVX000/EVIDENCE/ac-2.log)

### Define Hosted HTTP Control Contracts
- **ID:** 1vzEVi000
- **Status:** done

#### Summary
Define the shared hosted HTTP route, auth, and payload contracts so the CLI,
SDK, control plane, and node agent all speak one live hosted transport model.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Port defines implementation-ready hosted HTTP contracts for canonical machine and guest routes, including auth headers, request bodies, and response envelopes that the CLI, SDK, control plane, and node agent can share. <!-- [SRS-01/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzEVi000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-01/AC-02] The shared contracts preserve explicit node, host-group, runtime-owner, and future substrate context instead of hard-coding a one-off demo transport. <!-- [SRS-01/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzEVi000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzEVi000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzEVi000/EVIDENCE/ac-2.log)

### Implement Node Agent Serve Path
- **ID:** 1vzEVk000
- **Status:** done

#### Summary
Implement `port node-agent serve` so one configured node owns a runtime root and
serves authenticated machine plus guest operations for the hosted control plane.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] `port node-agent serve` runs an authenticated endpoint that serves machine inspection and guest operation routes by reusing Port's existing runtime-root and guest transport logic. <!-- [SRS-02/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzEVk000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-02/AC-02] Node-agent failures surface machine, node, runtime-root, and guest-socket context clearly enough for operator debugging. <!-- [SRS-02/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzEVk000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzEVk000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzEVk000/EVIDENCE/ac-2.log)

### Implement Control Plane Serve Path
- **ID:** 1vzEVm000
- **Status:** done

#### Summary
Implement `port control-plane serve` so authenticated clients can execute
hosted machine and guest routes through configured node-agent endpoints.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `port control-plane serve` authenticates hosted API requests and serves canonical machine and guest routes by forwarding them to the resolved node agent. <!-- [SRS-01/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzEVm000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-01/AC-02] Control-plane auth, routing, and unavailable-node failures surface explicit control-plane and node context instead of opaque transport errors. <!-- [SRS-01/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzEVm000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzEVm000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzEVm000/EVIDENCE/ac-2.log)

### Route Hosted CLI And SDK Through Live Transport
- **ID:** 1vzEVo000
- **Status:** done

#### Summary
Route hosted CLI and SDK operations through the live control-plane transport
instead of the current in-process hosted runtime-root shortcut.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] Hosted `port machine ...` and `port guest ...` commands execute through the live hosted HTTP path whenever a machine resolves to `hosted-control-plane` mode. <!-- [SRS-03/AC-01] verify: cargo test -q -p port-cli, proof: ac-1.log -->
- [x] [SRS-04/AC-02] `port-sdk`, CLI help, and operator output align with the live hosted routes and distinguish shipped transport from still-planned follow-on behavior. <!-- [SRS-04/AC-02] verify: cargo test -q -p port-sdk && cargo test -q -p port-cli, proof: ac-2.log -->

#### Implementation Insights
- **1w03m0000: Preserve Hosted Control Metadata After Node-Local Execution**
  - Insight: Reusing local runtime helpers inside the node agent will silently downgrade `MachineStatus`, `StopResult`, `MachineMonitorReport`, and `MachineTopReport` back to the local control contract unless the hosted route metadata is re-applied before the HTTP response is encoded.
  - Suggested Action: Keep a projection layer at the node-agent boundary that restores hosted control contracts and route context after any localized runtime call.
  - Applies To: `crates/port-runtime/src/hosted_control_plane.rs`, hosted lifecycle and monitoring handlers
  - Category: architecture

- **1w03m1000: Prove Transport Cutovers With Divergent Client And Server Configs**
  - Insight: The most reliable regression test for this class of change is to boot the long-lived server processes with the correct runtime root, then run the CLI or SDK against a second config whose hosted runtime root is intentionally wrong.
  - Suggested Action: Use split server/client configs in future hosted transport tests whenever the goal is to prove that the network path, not a local shortcut, is carrying the request.
  - Applies To: `crates/port-cli/tests/*hosted*`, `crates/port-runtime/src/lib.rs` hosted tests, future hosted SDK tests
  - Category: testing


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzEVo000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzEVo000/EVIDENCE/ac-2.log)



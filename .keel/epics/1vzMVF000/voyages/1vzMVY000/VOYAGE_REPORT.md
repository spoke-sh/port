# VOYAGE REPORT: Streamed Guest Control Transport

## Voyage Metadata
- **ID:** 1vzMVY000
- **Epic:** 1vzMVF000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 5/5 stories complete

## Implementation Narrative
### Publish Streamed Guest Workflow Surface
- **ID:** 1vzMXM000
- **Status:** done

#### Summary
Publish the streamed guest-session and hosted-transfer workflow across the CLI,
README, hosted docs, and SDK docs with recorded proof.

#### Acceptance Criteria
- [x] [SRS-05/AC-01] CLI help, README, `docs/hosted.md`, and `docs/sdk.md` describe the streamed guest-session and hosted-transfer workflow plus its explicit boundaries through the canonical Port command model. <!-- [SRS-05/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzMXM000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-05/AC-02] Recorded proof demonstrates the streamed PTY, log-follow, hosted copy, and hosted forward workflow through the canonical CLI and docs surfaces for a new operator. <!-- [SRS-05/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzMXM000/verify-ac-2.sh, proof: ac-2.log -->
- [x] [SRS-06/AC-03] Recorded proof demonstrates that the streamed transport rollout preserves the existing Firecracker standard, hosted PVM, and AVF workflows. <!-- [SRS-06/AC-03] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzMXM000/verify-ac-3.sh, proof: ac-3.log -->

#### Implementation Insights
- **1vzMXM000: Workflow-surface stories need proof that matches the published wording**
  - Insight: Doc-only acceptance is still fragile unless the proof scripts check the exact published keywords and pair them with executable workflow tests. The fastest way to keep these stories honest was to combine `rg`-based surface checks with targeted CLI/runtime tests for the workflows named in the docs.
  - Suggested Action: For future workflow-surface stories, write verify scripts that inspect the text and replay the referenced commands before submit.
  - Applies To: `.keel/stories/*/verify-ac-*.sh`, CLI help text, README and docs updates
  - Category: process


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzMXM000/EVIDENCE/ac-1.log)
- [ac-3.log](../../../../stories/1vzMXM000/EVIDENCE/ac-3.log)
- [ac-2.log](../../../../stories/1vzMXM000/EVIDENCE/ac-2.log)

### Define Streamed Guest Session Contract
- **ID:** 1vzMXN000
- **Status:** done

#### Summary
Define the shared protocol, hosted attach contract, and SDK surface for
streamed PTY, log-follow, copy, and forward so the implementation stories can
land on one canonical guest-control contract.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] The shared guest protocol and hosted route contract define streamed session lifecycle semantics for attach, payload, EOF, exit, and failure without introducing a second guest command family. <!-- [SRS-01/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzMXN000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-01/AC-02] The contract makes stream ownership and termination behavior explicit enough for CLI, runtime, node-agent, and SDK callers to implement deterministic cleanup. <!-- [SRS-01/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzMXN000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzMXN000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzMXN000/EVIDENCE/ac-2.log)

### Implement Streamed Pty And Log Follow
- **ID:** 1vzMXo000
- **Status:** done

#### Summary
Implement the stream-capable PTY and log-follow guest paths through the shared
guest protocol, guest agent, runtime, and canonical CLI.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] `port guest pty` provides streamed interactive behavior through the canonical CLI and shared guest protocol for local and AVF-backed runtimes. <!-- [SRS-02/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzMXo000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-02/AC-02] `port guest logs --follow` streams incremental guest log output while preserving the existing non-follow log behavior. <!-- [SRS-02/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzMXo000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzMXo000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzMXo000/EVIDENCE/ac-2.log)

### Implement Hosted Streamed Copy Transport
- **ID:** 1vzMXy000
- **Status:** done

#### Summary
Replace the hosted guest-copy bootstrap assumption with real streamed byte
transport through the control plane and node agent.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] Hosted `port guest copy` transfers bytes through the hosted control-plane and node-agent path without assuming the source or destination host paths are directly visible on the node host. <!-- [SRS-03/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzMXy000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-03/AC-02] Hosted copy success and failure paths surface explicit route, auth, and ownership context instead of ambiguous transport errors. <!-- [SRS-03/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzMXy000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzMXy000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzMXy000/EVIDENCE/ac-2.log)

### Implement Hosted Streamed Forward Transport
- **ID:** 1vzMY2000
- **Status:** done

#### Summary
Move hosted guest forwarding onto node-owned streamed transport so the hosted
path no longer depends on repo-local listener lifecycle.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] Hosted `port guest forward` uses a real hosted transport path owned by the control plane and node agent while preserving the canonical command family. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzMY2000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-04/AC-02] Hosted forward does not silently fall back to the repo-local listener lifecycle once the hosted machine resolves to a streamed transport owner. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzMY2000/verify-ac-2.sh, proof: ac-2.log -->

#### Implementation Insights
- **1vzMY2000: Hosted forward ownership can hide behind local listener setup**
  - Insight: The hosted control-plane path can be functionally live while the canonical CLI still bypasses it and silently falls back to local runtime assumptions. Forward ownership broke specifically because the CLI kept constructing a local session instead of entering the hosted `guest:forward` route.
  - Suggested Action: Add hosted-path tests that use a bogus client-side runtime root and require the control-plane/node-agent route to succeed or fail with hosted route context.
  - Applies To: `crates/port-runtime/src/lib.rs`, `crates/port-cli/src/lib.rs`, hosted guest capability tests
  - Category: architecture


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzMY2000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzMY2000/EVIDENCE/ac-2.log)



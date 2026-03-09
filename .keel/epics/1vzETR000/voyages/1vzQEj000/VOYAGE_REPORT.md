# VOYAGE REPORT: Hosted Detached Forward Lifecycle

## Voyage Metadata
- **ID:** 1vzQEj000
- **Epic:** 1vzETR000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 4/4 stories complete

## Implementation Narrative
### Define Hosted Detached Forward Contract
- **ID:** 1vzQIq000
- **Status:** done

#### Summary
Define the shared hosted route, payload, and SDK contract for detached guest
forward lifecycle operations so implementation stories can land on one
canonical API surface.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] The shared hosted contract defines detached forward start, list, and stop request/response shapes, including named session identity, without inventing a second guest command family. <!-- [SRS-01/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzQIq000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-01/AC-02] The detached forward contract preserves enough machine, node, runtime-root, and forward-name context for later routing and operator-facing failures. <!-- [SRS-01/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzQIq000/verify-ac-2.sh, proof: ac-2.log -->

#### Implementation Insights
- **1vzQKh000: Keel Story Record Proof Mapping Can Drift Across Same-SRS ACs**
  - Insight: `keel story record` can overwrite the inline `proof:` annotation for an earlier AC with the later AC's evidence file, while leaving the checkbox state unchanged.
  - Suggested Action: Inspect the story README after every multi-AC `story record` run and correct proof links or checkboxes before submit.
  - Applies To: `.keel/stories/*/README.md`, `keel story record` workflows
  - Category: process


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzQIq000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzQIq000/EVIDENCE/ac-2.log)

### Publish Hosted Detached Forward Workflow
- **ID:** 1vzQIy000
- **Status:** done

#### Summary
Publish the canonical hosted detached forward operator workflow across help
text, docs, and proof so the lifecycle commands are discoverable and runnable.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] CLI help, README, hosted docs, and SDK docs explain hosted detached `guest forward` start, list, stop, and `--name` behavior through the canonical Port surfaces. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzQIy000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-04/AC-02] The published workflow and proof make the hosted detached-forward boundary explicit enough that operators can tell what is shipped versus what remains follow-on work. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzQIy000/verify-ac-2.sh, proof: ac-2.log -->

#### Implementation Insights
- **1w04h0000: Keep Hosted Demo Socket Paths Short Under Nested Nix Shells**
  - Insight: Nested Nix shells can set `TMPDIR` to a long path that pushes Unix socket files past `SUN_LEN`, so demo proofs that rely on Unix sockets must pick a short temp root and avoid repeated `cargo run` startup races
  - Suggested Action: Use a fixed short `/tmp` workdir prefix and prebuild binaries before backgrounding guest-agent, node-agent, or control-plane demo processes
  - Applies To: scripts/hosted-demo.sh, hosted CLI proof scripts, Unix-socket integration tests
  - Category: testing


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzQIy000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzQIy000/EVIDENCE/ac-2.log)

### Implement Hosted Detached Forward Inventory
- **ID:** 1vzQJ6000
- **Status:** done

#### Summary
Implement the node-owned detached forward start, list, and stop behavior for
hosted machines so lifecycle state lives under the runtime-owning node instead
of the repo-local CLI.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] The node agent can start a detached hosted forward and return the resulting manifest summary from node-owned runtime state. <!-- [SRS-02/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzQJ6000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-02/AC-02] Hosted detached forward list and stop operate on node-owned manifests and clean up runtime artifacts without depending on repo-local CLI state. <!-- [SRS-02/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzQJ6000/verify-ac-2.sh, proof: ac-2.log -->

#### Implementation Insights
- **1vzQL2000: Detached Forward Runtime Helpers Must Not Assume `current_exe` Is The Port CLI**
  - Insight: `std::env::current_exe()` can resolve to a Rust test harness instead of the `port` CLI binary, so detached child-process launch must prefer an explicit or workspace `port` binary path before falling back to the current executable.
  - Suggested Action: Keep detached helper launchers behind a resolver that checks `PORT_DETACHED_FORWARD_EXECUTABLE` and the workspace `target/debug/port` path before using `current_exe()`.
  - Applies To: `crates/port-runtime/src/lib.rs`, detached runtime helpers, hosted node-agent tests
  - Category: architecture


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzQJ6000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzQJ6000/EVIDENCE/ac-2.log)

### Route Hosted Detached Forward Lifecycle
- **ID:** 1vzQJB000
- **Status:** done

#### Summary
Route hosted detached guest forward lifecycle actions through the canonical CLI
and SDK so hosted start, list, stop, and `--name` all use the live control
plane and node-agent path.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] Hosted `port guest forward --lifecycle detached [--name ...]` uses the live control-plane and node-agent path while preserving the existing command family. <!-- [SRS-03/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzQJB000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-03/AC-02] Hosted `port guest forward --list` and `--stop --name ...` use the live hosted transport and no longer fall back to repo-local lifecycle state. <!-- [SRS-03/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzQJB000/verify-ac-2.sh, proof: ac-2.log -->

#### Implementation Insights
- **1vzQLp000: Bogus Client Runtime Roots Are A Strong No-Fallback Hosted CLI Proof**
  - Insight: If the client config points the hosted node runtime root at a bogus path while the server-side config keeps the real runtime root, any successful hosted command proves the CLI is using remote transport rather than local state inspection.
  - Suggested Action: Keep using split server/client hosted configs with a bogus client runtime root in CLI integration tests for hosted transport stories.
  - Applies To: `crates/port-cli/tests/*`, hosted machine and guest transport tests
  - Category: testing


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzQJB000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzQJB000/EVIDENCE/ac-2.log)



# VOYAGE REPORT: Clarify Help Examples

## Voyage Metadata
- **ID:** 1vyfCm000
- **Epic:** 1vydg7000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 1/1 stories complete

## Implementation Narrative
### Fix Help Example Guidance
- **ID:** 1vyfD0000
- **Status:** done

#### Summary
Make the `port --help` examples explicit about their environment prerequisites
and runnable sequence, then align the supporting docs so operators understand
when `nix develop` and `port doctor` are required.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `port --help` states the prerequisite environment for the local artifact and launch examples and presents a runnable local workflow order. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && /tmp/port-target/debug/port --help | rg -n "nix develop|port doctor|machine launch|artifacts build"', proof: ac-1.log-->
- [x] [SRS-02/AC-01] README and operator docs explain the same prerequisite boundary as the CLI help. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "nix develop|port doctor|firecracker|PATH|machine launch" /home/alex/workspace/spoke-sh/port/README.md /home/alex/workspace/spoke-sh/port/docs/operators.md', proof: ac-2.log-->
- [x] [SRS-03/AC-01] The published help-example workflow is recorded with direct CLI evidence in the documented environment. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo run -p port-cli -- doctor && nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo run -p port-cli -- --config examples/port.toml artifacts build --artifact demo-kernel && nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo run -p port-cli -- --config examples/port.toml artifacts build --artifact demo-guest && nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo run -p port-cli -- --config examples/port.toml doctor', proof: ac-3.log-->

#### Implementation Insights
- **1vyfFY000: Say When Help Examples Depend On Repo Context**
  - Insight: Example commands that are syntactically correct can still feel broken if the help text does not state the repository-root assumption and the dependency environment right next to the examples.
  - Suggested Action: Put prerequisite and working-directory assumptions adjacent to CLI examples whenever commands depend on repo-relative files or external tools on PATH.
  - Applies To: `crates/port-cli/src/lib.rs`, CLI help text, operator docs
  - Category: documentation


#### Verified Evidence
- [ac-1.log](../../../../stories/1vyfD0000/EVIDENCE/ac-1.log)
- [ac-3.log](../../../../stories/1vyfD0000/EVIDENCE/ac-3.log)
- [ac-2.log](../../../../stories/1vyfD0000/EVIDENCE/ac-2.log)



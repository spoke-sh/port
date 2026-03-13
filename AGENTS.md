# AGENTS.md

Shared guidance for AI agents working with this repository. This file can be
imported by harness-specific files.

## Bootstrap Workflow

1. Enter the development shell with `nix develop`.
2. If the board is not initialized yet, run `just keel init` in the repo root.
3. Regenerate board summaries after structural or lifecycle changes with `just keel generate`.
4. Validate board health before finalizing work with `just doctor`.

## Search Tools

Fast search tools are available in the dev shell:

- `rg` for general code and file search.
- `sift` when file-type-aware or broader content filtering is useful.

## Mission Proof Contract

Treat the repo-level mission proof surface as the future answer to "can Port
host an application?" rather than only a board summary.

Current and planned naming:

- Today the repo-level proof entrypoint is `just mission`.
- The intended final product name is `screen`.
- Once upstream `keel screen` exists, replace the repo-local entrypoint with
  `just screen` routed through `just keel screen`.
- Treat that as a hard cutover. Do not keep `just mission` and `just screen`
  as parallel long-term surfaces unless a scoped story explicitly requires a
  migration window.

1. The canonical proof should launch a minimal HTTP application inside
   Port-managed compute, expose it through the canonical Port transport or
   forwarding surface, `curl` it from the host, and record the result through
   the Keel proof system.
2. Prefer human-reviewable terminal evidence for this path. Use the currently
   working recorder path first:
   - `vhs` or renderer-backed `.gif` / `.cast` artifacts today
   - `atxt` once it is stable and verified in this repository environment
3. Do not leave recorder migrations as chat-only follow-ups. If a better proof
   recorder is blocked on external tool maturity, create or maintain a routine
   that periodically reassesses readiness and materializes a scoped story when
   the tool becomes viable.
4. The current external-tool follow-up is `atxt`. Future agents should treat
   "migrate mission proof recording from `vhs` to `atxt`" as an explicit board
   commitment, not an optional idea. Maintain the routine
   `review-atxt-mission-proof-adoption` instead of opening duplicate reminder
   loops.
5. Until `keel screen` ships, keep the current repo-local `just mission`
   implementation working. When `keel screen` becomes available, prefer that
   Keel command over extending `scripts/mission-report.sh`.

## Subagent / Delegation

Use missions as the long-lived steering context and keep delivery contexts
narrow. If your harness supports subagents, worker sessions, or fresh
task-local contexts, use them to preserve workflow-specific focus instead of
carrying one mixed context across planning, research, and execution.

1. **Keep One Mission Steward**: The top-level harness/session owns mission
   scope, charter integrity, `just keel mission show <id>`, `just flow`,
   `just keel mission next [<id>]`, mission logging, phase switching, and
   final mission lifecycle transitions. Omit the ID to auto-select the
   highest-priority actionable mission.
2. **Delegate By Workflow Type**: Hand one concrete work unit to a dedicated
   worker context:
   - **Operator**: one primary implementation slice at a time, usually one
     story plus any directly coupled lifecycle work required to finish that
     slice cleanly, for example `story submit`, evidence capture, or
     `voyage done` when closing the final scoped story.
   - **Manager**: one planning unit at a time, including authored artifacts and
     downstream story decomposition needed to seal that unit cleanly.
   - **Explorer**: exactly one bearing research package, one lifecycle
     transition chain, one atomic commit.
3. **Pass Primary Sources, Not Just Summaries**: Give each worker the entity
   IDs, file ownership, verification expectations, lifecycle expectations, and
   the canonical `show` commands or document paths it must open first.
4. **Return Control After Each Unit**: When a worker finishes, the mission
   steward reviews the result, records the outcome with
   `just keel mission log <id> --entry "<text>"`, optionally runs
   `just keel mission digest <id>` for long logs, then reruns board health
   commands before choosing the next phase.
5. **Do Not Mix Phases In One Worker**: If the work changes from execution to
   planning or research, stop and hand off to the matching workflow context
   instead of continuing in the old one. Parent context reads and directly
   coupled closure steps are fine; silent mission re-scoping is not. Only
   parallelize workers when their artifacts and ownership do not overlap.

## Delivery Workflow (Operator)

**Operational Contract**: Focused operator for evidence-backed delivery.

Use this workflow inside a dedicated operator context. When operating under a
mission, the mission steward should hand one primary story slice at a time to
this workflow, along with any directly coupled parent context or closure work
needed to finish that slice cleanly.

1. **Pull Context**: Read current board health and identify bottlenecks with
   `just flow`.
2. **Claim Work**: Pull the highest-priority implementation item with
   `just keel next --role operator`. Use
   `just keel next --role operator --parallel` to identify safe concurrent
   tasks.
   - If no story is ready and the active mission still has unmet goals, hand
     off immediately to the Management or Research workflow instead of waiting
     or stopping.
3. **Open The Show Surfaces First**: Use the CLI read views as the default
   entry points for implementation context and clarification:
   - `just keel story show <story-id>` for the active work item, acceptance
     criteria, status, evidence, and story path.
   - `just keel voyage show <voyage-id>` for parent requirements, scope,
     progress, and rendered artifact paths.
   - `just keel epic show <epic-id>` for the problem statement, goals,
     requirement coverage, scope drift, and rendered artifact paths.
   - When you need full authored detail, follow the document paths shown in
     these views (`README.md`, `PRD.md`, `SRS.md`, `SDD.md`, and related
     artifacts) instead of guessing from summaries.
4. **Check Story Coherence Before Coding**: Confirm acceptance criteria are
   traceable and verifiable.
   - Acceptance criteria are linked to source requirements, for example
     `[SRS-XX/AC-YY]`.
   - Evidence strategy is clear for each criterion, for example test, CLI
     proof, or manual proof.
   - If requirements are ambiguous, loop back to the relevant `show` command
     and the linked planning artifacts before implementation.
5. **Execute (TDD)**: Follow test-driven development.
   - Write a failing test first.
   - Implement only enough to pass.
   - Refactor within the same change slice.
6. **Record Evidence**: Capture proof of requirement satisfaction for each
   acceptance criterion:
   - `just keel story record <ID> --ac <NUM> --cmd "<command>"`
   - For manual proofs, use `--msg`.
   - Use `--file` when the proof already exists as a human-reviewable artifact.
7. **Reflect Selectively (Optional)**: Use `just keel story reflect <ID>` only
   when the work uncovered a novel, reusable insight that is likely to help
   future stories. Reflection is optional and is no longer a mandatory gate for
   submission.
   - Start from the similar knowledge surfaced by the command. Prefer linking
     an existing knowledge file over creating a new one when the insight is
     already covered.
   - Capture only durable guidance another agent can reuse: a decision rule,
     failure mode, parser/rendering trap, verification lesson, or workflow
     guardrail.
   - Include the trigger or context, the reusable takeaway, and where it
     applies.
   - Do not record story recap, commit summary, obvious implementation steps,
     or one-off cleanup details already visible in the diff, proofs, or
     authored artifacts.
   - If there is no durable reusable insight, skip reflection.
8. **Submit**: Move to the human queue for review with
   `just keel story submit <ID>`. This triggers automated verification and
   generates the verification manifest. Resolve failures and rerun `submit`
   until the story reaches its post-submit state.
9. **Commit (Required)**: Create exactly one atomic
   [Conventional Commit](https://www.conventionalcommits.org/) for this story
   after `submit`, not before. Include the resulting `.keel` changes from
   submission in the same commit, for example story status updates, manifests,
   synthesized knowledge, and board projections. Do not batch multiple stories
   into one commit.

## Management Workflow (Manager)

**Operational Contract**: Scope, requirements, and decomposition steward.

Use this workflow inside a dedicated manager context. When operating under a
mission, enter this workflow only for a concrete planning unit that is blocked
on requirements, design, or decomposition.

1. **Identify Gaps & Maintain Architectural Integrity**: Use `just flow`
   to find epics needing tactical decomposition and to detect when delivery is
   starved by missing planning work.
   - When you need the management pull surface directly, use
     `just keel next --role manager`. Do not rely on `just keel next`; the
     role is required.
2. **Scaffold Planning Unit (Atomic Planning)**: Focus on one strategic or
   tactical unit at a time. Do not batch-create epics or voyages. Ensure one
   unit is fully authored and coherent before scaffolding the next.
   - For new strategic work, create an Epic:
     `just keel epic new "<Title>" --problem "<Problem>"`
   - For tactical decomposition, create a Voyage:
     `just keel voyage new "<Title>" --epic <epic-id> --goal "<Specific outcome>"`
3. **Author Epic PRD Immediately After Creation**: Before decomposing into
   voyages or stories, fill out `PRD.md` with authored content for every
   required section.
   - At minimum include problem statement, goals and objectives, users, scope,
     requirements, verification strategy, assumptions, open questions and
     risks, and success criteria.
   - Author goals with canonical `GOAL-*` rows.
   - Author scope with canonical `[SCOPE-*]` bullets.
   - Keep every PRD requirement linked to one or more valid `GOAL-*` IDs so
     goal coverage is explicit.
4. **Define Scope + Requirements (SRS)**: Fill out the voyage `SRS.md`.
   - In `## Scope`, map the parent epic scope with canonical `[SCOPE-*]`
     bullets so the voyage explicitly states what it takes on and what it
     defers.
   - Keep requirements atomic and uniquely identified, for example `SRS-01`.
   - The `Scope` column should use canonical `SCOPE-*` IDs declared by the
     voyage scope mapping.
   - The `Source` column should use exactly one parent PRD requirement ID
     (`FR-*` or `NFR-*`).
   - Write requirements so they map directly to story acceptance criteria and
     verification evidence.
5. **Detail Design (SDD)**: Fill out `SDD.md` with the architectural approach
   and component changes, with enough specificity that implementers can produce
   objective proofs.
6. **Decompose Stories**: Break the design into implementable units:
   - `just keel story new "<Title>" --type feat --epic <epic-id> --voyage <voyage-id>`
   - `--voyage` requires `--epic`. Omit both flags for unscoped stories, or
     pass only `--epic` for epic-scoped stories.
   - Link stories to SRS requirements using `[SRS-XX/AC-YY]` markers in the
     acceptance criteria.
7. **Align Verification Techniques From Config**: Run `just keel config show`,
   `just keel verify detect`, and `just keel verify recommend` before
   finalizing verification planning.
   - Use `just keel config show` as the full technique inventory and review
     each option's `detected`, `disabled`, and `active` flags.
   - Use `just keel verify detect` to review project signal detection inputs
     and per-technique detected and active status.
   - Use `just keel verify recommend` to plan against detected and active
     options for the current project.
   - If needed techniques are missing or disabled, update `keel.toml` first,
     then continue decomposition.
8. **Run Coherence Review**: Before planning is sealed, ensure:
   - Every PRD requirement row has valid goal links, and every authored
     `GOAL-*` is covered by at least one PRD requirement.
   - Voyage scope bullets and SRS `Scope` cells use canonical parent
     `SCOPE-*` IDs consistently.
   - Every SRS requirement has exactly one valid parent PRD source and at least
     one linked story acceptance criterion.
   - Every acceptance criterion has a concrete verification path.
   - Verification commands align with detected and recommended techniques
     unless explicitly justified.
   - CLI options and authored entity content are explicit enough for downstream
     automation and lifecycle transitions.
9. **Loop Back Upstream if Needed**: If decomposition exposes ambiguity, update
   PRD, SRS, or SDD first, then re-check story acceptance criteria.
10. **Generate Planning Summary In Chat (Required)**: Publish a terse planning
    summary in the harness response for every newly planned Epic or Voyage.
    Include objective and scope boundaries, requirement-to-story coverage
    status, verification strategy summary, key risks or assumptions, and the
    canonical next-step command.
11. **Seal Planning**: Promote the voyage from `draft` to `planned` with
    `just keel voyage plan <id>`.
12. **Commit (Required)**: Create exactly one atomic Conventional Commit for
    this planning unit after sealing so the resulting `.keel` state is included
    in the same commit.
13. **Return To Delivery**: After planning completes, immediately rerun
    `just keel next --role operator` and continue implementation unless a real
    blocker remains.

## Research Workflow (Explorer)

**Operational Contract**: Hypothesis-driven researcher for technical discovery
and fog reduction.

Use this workflow inside a dedicated explorer context. When operating under a
mission, enter this workflow only when ambiguity or missing evidence blocks
planning or execution.

1. **Identify Fog**: Create a bearing when ambiguity or missing evidence blocks
   planning or execution:
   - `just keel bearing new "<Name>"`
   - Always use the CLI scaffold. A bearing should include `README.md`,
     `BRIEF.md`, `EVIDENCE.md`, and `ASSESSMENT.md`.
2. **Discovery (Play)**: Use `just keel play <id>` to explore the problem space
   through different perspectives.
3. **Draft Brief**: Fill out `BRIEF.md`. The sections `Hypothesis`,
   `Problem Space`, `Success Criteria`, and `Open Questions` are mandatory.
4. **Research Evidence**: Document source-backed findings, constraints, and
   unknowns in `EVIDENCE.md`.
5. **Seal Research**: Transition to the research phase with
   `just keel bearing research <id>`.
6. **Assess Impact**: Document recommendation and impact in `ASSESSMENT.md`.
7. **Seal Assessment**: Transition to the assessing phase with
   `just keel bearing assess <id>`.
8. **Graduate**: If research is conclusive, graduate the bearing to a strategic
   epic with `just keel bearing lay <id>`.
9. **Commit (Required)**: Create exactly one atomic Conventional Commit for the
   research package after the final lifecycle transition you take for it.
10. **Feed The Next Phase Immediately**: After assessment or graduation,
    immediately create or update the downstream epic, voyage, or stories, or
    hand control back to delivery in the same mission loop.

## Mission Workflow (Autonomous Harness / Mission Steward)

Missions are the top-level steering loop for broad objectives. Once a mission
exists, the harness should keep working the mission until its halting rules are
met or a real external blocker is reached.

Broad product or feature goals should always run inside an active mission. If
the request is broad and no mission covers it yet, create one first.

1. **Bootstrap Mission**: If no active mission covers the current user request,
   create one:
   - `just keel mission new "<Title>"`
2. **Refine Charter**: Fill out `CHARTER.md` with specific goals, constraints,
   and halting rules before activation.
   - Every mission goal should have a clear verification path: `board:`,
     `metric:`, or `manual:`.
   - Use `just keel mission refine <id>` to see the next question.
   - Use `just keel mission refine <id> --answer "<text>"` to record each
     answer.
   - Do not activate the mission until refinement reports that the charter is
     ready and every authored goal has an explicit verification path.
3. **Activate**: Once the charter is ready and the mission has actionable
   board-linked goals, activate it:
   - `just keel mission activate <id>`
4. **Refresh State At The Start Of Every Cycle**:
   - `just keel mission show <id>`
   - `just flow`
   - `just keel mission next [<id>]` to resolve indecision points across all
     roles. Omit the ID to auto-select the highest-priority actionable mission.
5. **Choose The Correct Phase And Hand Off To The Matching Workflow**:
   - Use `just keel mission next [<id>]` to see the immediate priority for all
     role families. Omit the ID to auto-select the highest-priority actionable
     mission.
   - If a story is ready or the next step is a concrete implementation slice,
     use the Delivery Workflow in a dedicated operator context.
   - If no story is ready and the next step is decomposition, scoping, or
     requirements or design authoring, use the Management Workflow in a
     dedicated manager context.
   - If planning or execution is blocked by ambiguity, missing evidence, or
     external research, use the Research Workflow in a dedicated explorer
     context.
   - If `just keel mission next [<id>]` reports no ready work but mission
     goals remain unmet, create the next bearing, epic, voyage, or story
     instead of stopping.
6. **Rejoin The Mission Loop After Every Worker Result**:
   - Review the resulting board state and changed artifacts.
   - Record the decision, evidence, blocker, and next phase with
     `just keel mission log <id> --entry "<text>"`.
   - Return to step 4 and choose the next phase again. Do not let one
     long-lived worker drift across multiple workflow types.
7. **Digest Regularly**: When `LOG.md` grows large, compress older entries:
   - `just keel mission digest <id>`
8. **Achieve**: Once all board-verifiable goals are satisfied:
   - `just keel mission achieve <id>`
9. **Final Verification**: After achievement:
   - `just keel mission verify <id>`
10. **Non-Stopping Conditions**: The following are never sufficient reasons to
    halt while mission goals remain unmet:
   - no ready story
   - clean tests
   - a clean commit
   - a submitted story

## Global Hygiene Checklist

Apply these checks to every change before finalizing work:

1. **Doctor Check**: `just doctor` must pass with zero warnings or errors.
2. **Quality Check**: `just check` must pass.
3. **Verification**: `just test` and `just doctest` must pass when relevant to
   the change.
4. **Lifecycle Before Commit**: Run board-mutating lifecycle commands before
   the atomic commit when they generate or rewrite `.keel` artifacts, for
   example `story submit`, `voyage plan`, `voyage done`, `bearing assess`, or
   `bearing lay`. After the transition, inspect `git status` and include the
   resulting `.keel` churn in the same commit.
5. **Atomic Commits**: Commit once per logical unit of work using
   [Conventional Commits](https://www.conventionalcommits.org/).
6. **Mission Loop Discipline**: For mission-driven work, return to the mission
   steward loop after every completed story, planning unit, or bearing instead
   of continuing ad hoc from the last worker context.
7. **Knowledge Quality Bar**: Prefer no new knowledge over low-signal
   knowledge. New knowledge should be novel, reusable across stories, and
   materially reduce future drift.

## Compatibility Policy (Hard Cutover)

At this stage of development, this repository uses a hard cutover policy by
default.

1. **No Backward Compatibility by Default**: Do not add compatibility aliases,
   dual-write logic, soft-deprecated schema fields, or fallback parsing for
   legacy formats unless a story explicitly requires it.
2. **Replace, Don’t Bridge**: When introducing a new canonical token, field,
   command behavior, or document contract, remove the old path in the same
   change slice.
3. **Fail Fast in Validation**: `keel doctor` and transition gates should treat
   legacy or unfilled scaffold patterns as hard failures when they violate the
   current contract.
4. **Single Canonical Path**: Keep one source of truth for rendering, parsing,
   and validation.
5. **Migration Is Explicit Work**: If existing artifacts need updates, handle
   that in a dedicated migration pass instead of embedding runtime compatibility
   logic.

## Foundational Documents

These define current constraints and workflow:

- `README.md` — repository intent and high-level project statement.
- `AGENTS.md` — shared agent workflow contract.
- `flake.nix` — development shell and tool entrypoint.
- `justfile` — repo-local build, test, and board command wrapper surface.
- `.keel/adrs/` — binding architecture decisions when present.
- `.keel/` planning artifacts — executable requirements, design, and work
  breakdown.

Use this order when interpreting constraints: ADRs, then `README.md`, then
repo-local workflow docs and planning artifacts.

## Project Overview

This repository is `port` — agentic compute orchestration in Firecracker VMs.

The repository is still evolving, but the board should remain authoritative for
sequencing, evidence, and traceability.

| Path | Purpose |
|------|---------|
| `README.md` | Current project description |
| `flake.nix` | Nix flake for the dev shell and shared tooling |
| `justfile` | Repo-local workflow wrappers |
| `AGENTS.md` | Shared agent workflow contract |
| `.keel/` | Project board, planning artifacts, and ADRs |

## Board Directory (`.keel/`)

A `.keel/` directory is the runtime data directory that `keel` operates on.

| Path | Contents |
|------|----------|
| `.keel/adrs/` | Architecture decision records |
| `.keel/epics/` | Epic-level planning artifacts |
| `.keel/epics/<epic-id>/voyages/` | Voyage planning artifacts |
| `.keel/routines/` | Recurring routine bundles |
| `.keel/stories/` | Implementable work items |
| `.keel/README.md` | Board state overview |

## Commands

### Command Execution Model

Use one path for each concern:

- `nix develop` for the repository shell and shared tooling.
- `just ...` for repo build, test, formatting, and helper workflows.
- `just keel ...` for all planning, mission, execution, research, and
  verification workflows.

### `just` Workflow Commands

| Command | Purpose |
|---------|---------|
| `just` | List available recipes |
| `just fmt` | Format the workspace |
| `just fmt-check` | Check formatting |
| `just clippy` | Run workspace clippy |
| `just check` | Run the repo's quality gate |
| `just test [args]` | Run tests |
| `just doctest [args]` | Run doc tests |
| `just coverage [args]` | Produce coverage output |
| `just port ...` | Run the Port CLI |
| `just next` | Convenience alias for `keel next --role operator` |

### `just keel` Board Workflow Commands

Run `just keel --help` for the full command tree. Common commands:

| Category | Commands |
|----------|----------|
| Discovery | `just keel bearing new <name>` `just keel play <id>` `just keel bearing research <id>` `just keel bearing assess <id>` `just keel bearing list` |
| Planning | `just keel epic new "<name>" --problem "<problem>"` `just keel voyage new "<name>" --epic <epic-id> --goal "<goal>"` |
| Execution | `just keel next --role operator` `just keel story new "<title>" [--type <type>] [--epic <epic-id> [--voyage <voyage-id>]]` |
| Board Ops | `just keel mission next [<id>]` `just keel next --role manager` `just flow` `just doctor` `just keel generate` `just keel config show` `just keel mission show <id>` |
| Routines | `just keel routine new "<name>"` `just keel routine list` `just keel routine show <id>` `just keel pulse` |
| Verification | `just keel verify run <id>` `just keel verify detect` `just keel verify recommend` |

## Story and Milestone State Changes

Use CLI commands only. Do not move `.keel` files manually.

| Action | Command |
|--------|---------|
| Start | `just keel story start <id>` |
| Reflect | `just keel story reflect <id>` |
| Submit | `just keel story submit <id>` |
| Reject | `just keel story reject <id> "reason"` |
| Accept | `just keel story accept <id> --role manager` |
| Ice | `just keel story ice <id>` |
| Thaw | `just keel story thaw <id>` |
| Voyage plan | `just keel voyage plan <id>` |
| Voyage done | `just keel voyage done <id>` |
| Bearing assess | `just keel bearing assess <id>` |
| Bearing lay | `just keel bearing lay <id>` |
| Mission activate | `just keel mission activate <id>` |
| Mission achieve | `just keel mission achieve <id>` |

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

## Subagent / Delegation

Use missions as the long-lived steering context and keep delivery contexts
narrow. If your harness supports subagents, worker sessions, or fresh
task-local contexts, use them to preserve workflow-specific focus instead of
carrying one mixed context across planning, research, and execution.

1. **Keep One Mission Steward**: The top-level harness/session owns mission
   scope, charter integrity, `just keel mission show <id>`, `just flow`,
   `just keel mission next <id>`, mission logging, phase switching, and final
   mission lifecycle transitions.
2. **Delegate By Workflow Type**: Hand one concrete work unit to a dedicated
   worker context:
   - **Operator**: one primary implementation slice at a time, usually one
     story plus any directly coupled lifecycle work required to finish that
     slice cleanly.
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
   instead of continuing in the old one. Only parallelize workers when their
   artifacts and ownership do not overlap.

## Delivery Workflow (Operator)

**Operational Contract**: Focused operator for evidence-backed delivery.

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
4. **Check Story Coherence Before Coding**: Confirm acceptance criteria are
   traceable and verifiable.
   - Acceptance criteria are linked to source requirements, for example
     `[SRS-XX/AC-YY]`.
   - Evidence strategy is clear for each criterion.
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
7. **Reflect Selectively (Optional)**: Use `just keel story reflect <ID>` only
   when the work uncovered a novel, reusable insight that is likely to help
   future stories. Reflection is optional and is no longer a mandatory gate for
   submission.
   - Prefer linking existing knowledge over creating new low-signal knowledge.
   - If there is no durable reusable insight, skip reflection.
8. **Submit**: Move to the human queue for review with
   `just keel story submit <ID>`. This triggers automated verification and
   generates the verification manifest. Resolve failures and rerun `submit`
   until the story reaches its post-submit state.
9. **Commit (Required)**: Create exactly one atomic
   [Conventional Commit](https://www.conventionalcommits.org/) for this story
   after `submit`, not before. Include the resulting `.keel` changes from
   submission in the same commit.

## Management Workflow (Manager)

**Operational Contract**: Scope, requirements, and decomposition steward.

1. **Identify Gaps & Maintain Architectural Integrity**: Use `just flow`
   to find epics needing tactical decomposition and to detect when delivery is
   starved by missing planning work.
2. **Scaffold Planning Unit (Atomic Planning)**: Focus on one strategic or
   tactical unit at a time.
   - For new strategic work, create an Epic:
     `just keel epic new "<Title>" --problem "<Problem>"`
   - For tactical decomposition, create a Voyage:
     `just keel voyage new "<Title>" --epic <epic-id> --goal "<Specific outcome>"`
3. **Author Epic PRD Immediately After Creation**: Before decomposing into
   voyages or stories, fill out `PRD.md` with authored content for every
   required section.
4. **Define Scope + Requirements (SRS)**: Fill out the voyage `SRS.md`.
   Requirements must be atomic, uniquely identified, and directly traceable to
   story acceptance criteria.
5. **Detail Design (SDD)**: Fill out `SDD.md` with the architectural approach
   and component changes.
6. **Decompose Stories**: Break the design into implementable units:
   - `just keel story new "<Title>" --type feat --epic <epic-id> --voyage <voyage-id>`
   - `--voyage` requires `--epic`. Omit both flags for unscoped stories, or
     pass only `--epic` for epic-scoped stories.
7. **Align Verification Techniques From Config**: Run `just keel config show`,
   `just keel verify detect`, and `just keel verify recommend` before
   finalizing verification planning.
8. **Run Coherence Review**: Before planning is sealed, ensure:
   - Every parent requirement has downstream coverage.
   - Every acceptance criterion has a concrete verification path.
   - Verification commands align with detected and recommended techniques unless
     explicitly justified.
9. **Loop Back Upstream if Needed**: If decomposition exposes ambiguity, update
   PRD, SRS, or SDD first, then re-check story acceptance criteria.
10. **Generate Planning Summary In Chat (Required)**: Publish a terse planning
    summary in the harness response for every newly planned Epic or Voyage.
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

1. **Identify Fog**: Create a bearing when ambiguity or missing evidence blocks
   planning or execution:
   - `just keel bearing new "<Name>"`
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

Missions are the top-level steering loop for broad objectives. If the request
is broad and no mission covers it yet, create one first.

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
3. **Activate**: Once the charter is ready and the mission has actionable
   board-linked goals, activate it:
   - `just keel mission activate <id>`
4. **Refresh State At The Start Of Every Cycle**:
   - `just keel mission show <id>`
   - `just flow`
   - `just keel mission next <id>`
5. **Choose The Correct Phase And Hand Off To The Matching Workflow**:
   - Use `just keel mission next <id>` to see the immediate priority for all
     role families.
   - If a story is ready, use the Delivery Workflow in an operator context.
   - If no story is ready and the next step is decomposition or authored
     planning work, use the Management Workflow in a manager context.
   - If planning or execution is blocked by ambiguity or missing evidence, use
     the Research Workflow in an explorer context.
   - If `just keel mission next <id>` reports no ready work but mission goals
     remain unmet, create the next bearing, epic, voyage, or story instead of
     stopping.
6. **Rejoin The Mission Loop After Every Worker Result**:
   - Review the resulting board state and changed artifacts.
   - Record the outcome with
     `just keel mission log <id> --entry "<text>"`.
   - Return to the mission loop and choose the next phase again.
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
   `bearing lay`.
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
| Discovery | `just keel bearing new <name>` `just keel bearing research <id>` `just keel bearing assess <id>` `just keel bearing list` |
| Planning | `just keel epic new "<name>" --problem "<problem>"` `just keel voyage new "<name>" --epic <epic-id> --goal "<goal>"` |
| Execution | `just keel next --role operator` `just keel story new "<title>" [--type <type>] [--epic <epic-id> [--voyage <voyage-id>]]` |
| Board Ops | `just keel mission next <id>` `just flow` `just doctor` `just keel generate` `just keel config show` `just keel mission show <id>` |
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

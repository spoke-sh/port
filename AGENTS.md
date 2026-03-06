# AGENTS.md

Shared guidance for AI agents working with this repository.

## Bootstrap Workflow

1. Enter the development shell with `nix develop`.
2. If the board is not initialized yet, run `keel init` in the repo root.
3. Regenerate board summaries after structural changes with `keel generate`.
4. Validate board health before finalizing work with `keel doctor`.

## Autonomous Delivery Policy

These rules define how agents should behave when the user asks for end-to-end
product work rather than a single bounded change.

1. **Top-Level Objective Overrides Local Scope**: If the user provides a larger
   MVP, product objective, or end-state than the current board covers, treat
   that objective as the real stopping condition. The current story, voyage, or
   epic is only an intermediate slice.
2. **Queue Empty Is Not Completion**: If `keel next --agent` returns no ready
   work and the user objective is still incomplete, do not stop. Inspect the
   current epic, voyage, PRD, SRS, SDD, and README context, then create the
   next bearing, epic, voyage, or stories needed to continue.
3. **Voyage Completion Is Not Request Completion**: Finishing a voyage does not
   mean the user request is complete unless the user explicitly scoped the work
   to that voyage.
4. **Board Is Authoritative, But Not Self-Terminating**: Use the board as the
   system of record for sequencing, evidence, and traceability, but extend it
   whenever the active product objective still has unimplemented work.
5. **Stop Only For Real Boundaries**: Stop only when one of the following is
   true:
   - the defined MVP or user objective is complete,
   - a real external blocker prevents safe progress,
   - or the user explicitly asks to pause or change direction.
6. **Autonomous Manual Review**: If a story is blocked only on manual
   verification and the agent has directly inspected the relevant output, docs,
   or behavior, record the proof, submit the story, and complete the required
   human acceptance step explicitly with the appropriate `keel` command.
7. **Successive Voyages Are Expected**: In autonomous product-building mode,
   agents should expect to create and complete multiple voyages in sequence
   until the requested MVP is satisfied.

## Execution Workflow (Implementer)

1. **Pull Context**: Read current board health and identify bottlenecks with `keel flow`.
2. **Claim Work**: Pull the highest-priority implementation item with `keel next --agent`.
   - If no story is ready and the product objective is still incomplete, switch
     to research or planning work immediately instead of stopping.
3. **Check Story Coherence Before Coding**: Confirm acceptance criteria are traceable and verifiable:
   - Acceptance criteria are linked to source requirements (for example `[SRS-XX/AC-YY]`).
   - Evidence strategy is clear for each criterion (test, CLI proof, or manual proof).
   - If requirements are ambiguous, loop back to planning artifacts before implementation.
4. **Execute (TDD)**: Follow test-driven development:
   - Write a failing test first.
   - Implement only enough to pass.
   - Refactor within the same change slice.
5. **Record Evidence**: Capture proof of requirement satisfaction for each acceptance criterion:
   - `keel story record <ID> --ac <NUM> --msg "Description of the proof"`
   - For command-based proofs, use `--cmd`.
   - For manual proofs, use `--msg` or attached files.
6. **Reflect**: Run `keel story reflect <ID>` and document what was learned during implementation.
7. **Commit (Required)**: Create exactly one atomic [Conventional Commit](https://www.conventionalcommits.org/) for this story before submission.
8. **Submit**: Use `keel story submit <ID>` to run the transition gate.
   - If the story auto-completes, continue to the next item.
   - If the story requires manual verification and you have directly performed
     that review, complete the acceptance step explicitly and keep moving.
9. **Continue The Product**: After each completed story, re-run `keel flow` and
   `keel next --agent`. If the queue is empty but the user objective is not,
   extend the board and continue.

## Planning Workflow (Architect)

1. **Identify Gaps**: Use `keel flow` or `keel status` to find epics needing tactical decomposition.
   - If execution is starved but the user objective is incomplete, create the
     next planning unit instead of waiting for more instructions.
2. **Scaffold Planning Unit**:
   - For new strategic work, create an Epic: `keel epic new "<Title>" --goal "<Outcome>"`
   - For tactical decomposition, create a Voyage: `keel voyage new "<Title>" --epic <epic-id> --goal "<Specific outcome>"`
3. **Author Epic PRD Immediately After Creation**: Before decomposing into voyages or stories, fill out `PRD.md` with authored content for every required section.
4. **Define Requirements (SRS)**: Fill out the voyage `SRS.md`. Requirements should be atomic, uniquely identified, and directly traceable to story acceptance criteria.
5. **Detail Design (SDD)**: Fill out `SDD.md` with the architectural approach and component changes.
6. **Decompose Stories**: Break the design into implementable units with `keel story new "<Title>"`.
7. **Align Verification Techniques From Config**: Run `keel config show`, `keel verify detect`, and `keel verify recommend` before finalizing verification planning.
8. **Run Coherence Review**: Ensure every requirement has story coverage and every acceptance criterion has a concrete verification path.
9. **Loop Back Upstream if Needed**: If decomposition exposes ambiguity, update PRD, SRS, or SDD first.
10. **Generate Planning Summary In Chat (Required)**: Publish a terse planning summary in the harness response for every newly planned Epic or Voyage.
11. **Commit (Required)**: Create exactly one atomic [Conventional Commit](https://www.conventionalcommits.org/) for the planning unit.
12. **Seal Planning**: Promote the voyage when planning is complete with `keel voyage plan <id>`.
13. **Return To Execution**: After planning, immediately resume `keel next --agent`
    and continue implementation unless a real blocker remains.

## Research Workflow (Explorer)

1. **Identify Fog**: Create a bearing when the path forward is ambiguous: `keel bearing new "<Name>"`.
   - If execution is blocked by architectural uncertainty and the queue has no
     safe next step, create the bearing yourself and keep moving.
2. **Discovery (Play)**: Use `keel play <id>` to explore the problem space through different perspectives.
3. **Draft Brief**: Fill out `BRIEF.md`. The sections `Hypothesis`, `Problem Space`, `Success Criteria`, and `Open Questions` are mandatory.
4. **Survey Findings**: Document research, technical constraints, and alternatives in `SURVEY.md`.
5. **Seal Survey**: Transition to surveying with `keel bearing survey <id>`.
6. **Assess Impact**: Document the recommendation in `ASSESSMENT.md`.
7. **Seal Assessment**: Transition to assessment with `keel bearing assess <id>`.
8. **Commit (Required)**: Create exactly one atomic [Conventional Commit](https://www.conventionalcommits.org/) for the research package.
9. **Graduate**: If research is conclusive, graduate the bearing with `keel bearing lay <id>`.
10. **Feed Planning Or Execution Immediately**: Once the bearing is conclusive,
    create or update the downstream epic, voyage, or stories in the same
    overall workstream rather than stopping at research.

## Global Hygiene Checklist

Apply these checks to every change before finalizing work:

1. **Doctor Check**: `keel doctor` must pass with zero warnings or errors.
2. **Project Verification**: Run the repo-specific formatting, linting, and test commands that exist. If automation is not available yet, state exactly what was and was not verified.
3. **Board Regeneration**: Run `keel generate` after structural board changes so summaries stay current.
4. **Atomic Commits**: Commit once per logical unit of work using [Conventional Commits](https://www.conventionalcommits.org/).

## Compatibility Policy (Hard Cutover)

At this stage of development, this repository uses a hard cutover policy by default.

1. **No Backward Compatibility by Default**: Do not add compatibility aliases, dual-write logic, soft-deprecated schema fields, or fallback parsing for legacy formats unless a story explicitly requires it.
2. **Replace, Don’t Bridge**: When introducing a new canonical token, field, command behavior, or document contract, remove the old path in the same change slice.
3. **Fail Fast in Validation**: `keel doctor` and transition gates should treat legacy or unfilled scaffold patterns as hard failures when they violate the current contract.
4. **Single Canonical Path**: Keep one source of truth for rendering, parsing, and validation.
5. **Migration Is Explicit Work**: If existing artifacts need updates, handle that in a dedicated migration pass instead of embedding runtime compatibility logic.

## Foundational Documents

These define current constraints and workflow:

- `README.md` — repository intent and high-level project statement.
- `flake.nix` — Nix development environment and tool entrypoint.
- `.keel/adrs/` — binding architecture decisions once the board is initialized.
- `.keel/` planning artifacts — executable requirements, design, and work breakdown.

Use this order when interpreting constraints: ADRs, when present, then `README.md`, then planning artifacts.

## Project Overview

This repository is `port` — agentic compute orchestration in Firecracker VMs.

The repository is currently in bootstrap mode. Expect foundational documents, board artifacts, and runtime code to grow together; keep the board authoritative as soon as it exists.

| Path | Purpose |
|------|---------|
| `README.md` | Current project description |
| `flake.nix` | Nix flake for the dev shell and shared tooling |
| `AGENTS.md` | Shared agent workflow contract |
| `.keel/` | Project board, planning artifacts, and ADRs |

## Board Directory (`.keel/`)

A `.keel/` directory is the runtime data directory that `keel` operates on. It lives in this repository once initialized.

| Path | Contents |
|------|----------|
| `.keel/adrs/` | Architecture decision records |
| `.keel/epics/` | Epic-level planning artifacts |
| `.keel/epics/<epic-id>/voyages/` | Voyage planning artifacts (`SRS.md`, `SDD.md`) |
| `.keel/stories/` | Implementable work items |
| `.keel/README.md` | Board state overview |

## Commands

### Command Execution Model

Use one path for each concern:

- `nix develop` for the repository shell and shared tooling.
- `keel ...` for all planning, execution, research, and verification workflows.
- Project-specific build and test commands should be documented in `README.md` or a future `justfile` as the codebase lands.

### Core `keel` Commands

| Category | Commands |
|----------|----------|
| Setup | `keel init` `keel config show` `keel generate` |
| Discovery | `keel bearing new <name>` `keel bearing survey <id>` `keel bearing assess <id>` `keel bearing list` |
| Planning | `keel epic new <name> --goal <goal>` `keel voyage new <name> --epic <epic-id> --goal <goal>` |
| Execution | `keel next --agent` `keel story new <title>` `keel story start <id>` `keel story record <id>` `keel story reflect <id>` `keel story submit <id>` |
| Diagnostics | `keel doctor` `keel status` `keel flow` `keel gaps` `keel verify detect` `keel verify recommend` |

# AGENTS.md

Shared guidance for AI agents working with this repository.

## Operational Guidance

This repository uses Keel as its project management engine. Your primary responsibility is to execute tactical moves that advance the board state while maintaining 100% integrity.

### Core Principles
1. **Gardening First**: You MUST tend to the garden (fixing `doctor` errors, discharging automated backlog, and resolving structural drift) BEFORE notifying the human operator or requesting input. 
2. **Heartbeat Hygiene**: Monitor the system's pulse via `keel heartbeat` and `keel health --scene`. The pacemaker is derived from repository activity; uncommitted energy in the worktree is tactical debt that should be closed autonomously by landing the sealing commit.
3. **Notification Discipline**: Ping the human operator ONLY when you need input on design direction or how the application behaves. Resolve technical drift and tactical moves autonomously.

### Canonical Turn Loop
Port's operator rhythm follows Keel's `Orient -> Inspect -> Pull -> Ship -> Close` loop surfaced by `keel turn`.

- **Orient**: Inspect charge and board stability with `keel heartbeat`, `keel health --scene`, `keel flow --scene`, and `keel doctor`.
- **Inspect**: Read current demand with `keel mission next --status`, `keel pulse`, `keel roles`, and `keel next --role <role> --explain` when routing is unclear.
- **Pull**: Select one role-scoped slice with `keel next --role <role>`.
- **Ship**: Execute the slice, record proof, and advance lifecycle state.
- **Close**: Land the relevant transition and the sealing commit that clears open-loop energy.

### Delivery And Git Discipline
The turn loop is a scheduling model. It does not relax Git hygiene, story boundaries, or completion semantics.

1. **One Story, One Clean Closure**: Finish one implementation story at a time. Record proof, land the relevant `.keel` lifecycle changes, and make at least one sealing commit for that story before pulling the next one.
2. **No Multi-Story Dirty Waves**: Do not let several stories accumulate in one dirty worktree. If the work no longer fits one story cleanly, stop and re-scope through planning instead of continuing ad hoc.
3. **No Completion Claims Before Commit**: Do not describe a story, voyage, epic, or mission as complete, executed, achieved, verified, or sealed while the relevant code or `.keel` changes are still uncommitted.
4. **Human Status Must Match Repo Reality**: Distinguish clearly between `implemented but uncommitted`, `committed and submitted`, and `accepted/sealed`. Do not collapse those states in chat.
5. **Pacemaker Guidance Is Not A Waiver**: Heartbeat, pacemaker, and loop-closure instructions mean "commit promptly." They never justify skipping story-level commit boundaries or claiming closure early.

### Session Start & Human Interaction
When a human user opens the chat or "pokes" you (e.g., "Wake up", "I'm poking you"), you MUST immediately perform the `Orient` and `Inspect` halves of the turn loop by following the **Human Interaction & Pokes** workflow in [INSTRUCTIONS.md](INSTRUCTIONS.md):
1.  **Heartbeat**: Run `keel heartbeat` to inspect current charge and whether the worktree is carrying uncommitted energy.
2.  **Pulse**: Run `keel health --scene` to check subsystem stability.
3.  **Scan**: Run `keel mission next --status` and `keel pulse`.
4.  **Confirm**: Run `keel flow --scene` to verify whether the LIGHT IS ON or the board is idle waiting for fresh repository activity.
5.  **Diagnose**: Run `keel doctor` to ensure board integrity before proceeding.

### Procedural Instructions
Follow the formal procedural loops and checklists defined in:
👉 **[INSTRUCTIONS.md](INSTRUCTIONS.md)**

## Mission Proof Contract

Treat the repo-level mission proof surface as the future answer to "can Port host an application?" rather than only a board summary.

Current and planned naming:

- Today the canonical text-based entrypoint is `keel mission show <id>`.
- `just mission [<id>]` is a thin repo-local convenience wrapper that auto-selects the current active mission or context when no ID is provided.
- `keel screen <id>` provides a **visual representation** of the project state, and `just screen [<id>]` is the corresponding convenience wrapper.
- Treat that as a hard cutover. The legacy `scripts/mission-report.sh` has been removed.

1. The canonical proof should launch a minimal HTTP application inside Port-managed compute, expose it through the canonical Port transport or forwarding surface, `curl` it from the host, and record the result through the Keel proof system.
2. Prefer human-reviewable terminal evidence for this path. Use the currently working recorder path first:
   - `vhs` or renderer-backed `.gif` / `.cast` artifacts today
   - `atxt` once it is stable and verified in this repository environment
3. Do not leave recorder migrations as chat-only follow-ups. If a better proof recorder is blocked on external tool maturity, create or maintain a routine that periodically reassesses readiness and materializes a scoped story when the tool becomes viable.
4. The current external-tool follow-up is `atxt`. Future agents should treat "migrate mission proof recording from `vhs` to `atxt`" as an explicit board commitment, not an optional idea. Maintain the routine `VEz56fPp4` instead of opening duplicate reminder loops.
5. Prefer the Keel mission show output directly over manual scripts. Use `keel mission show <id>` or `keel screen <id>` as the canonical surfaces.

## Subagent / Delegation

Use missions as the long-lived steering context and keep delivery contexts narrow. If your harness supports subagents, worker sessions, or fresh task-local contexts, use them to preserve workflow-specific focus instead of carrying one mixed context across planning, research, and execution.

1. **Keep One Mission Steward**: The top-level harness/session owns mission scope, charter integrity, `keel mission show <id>`, `keel flow`, `keel mission next [<id>]`, mission logging, phase switching, and final mission lifecycle transitions. Omit the ID to auto-select the highest-priority actionable mission.
2. **Delegate By Workflow Type**: Hand one concrete work unit to a dedicated worker context:
   - **Operator**: one primary implementation slice at a time, usually one story plus any directly coupled lifecycle work required to finish that slice cleanly, for example `story submit`, evidence capture, or `voyage done` when closing the final scoped story. The worker returns only after the slice is commit-ready or committed; do not hand back half-finished multi-story dirt.
   - **Manager**: one planning unit at a time, including authored artifacts and downstream story decomposition needed to seal that unit cleanly.
   - **Explorer**: exactly one bearing research package, one lifecycle transition chain, one atomic commit.
3. **Pass Primary Sources, Not Just Summaries**: Give each worker the entity IDs, file ownership, verification expectations, lifecycle expectations, and the canonical `show` commands or document paths it must open first.
4. **Return Control After Each Unit**: When a worker finishes, the mission steward reviews the result, records the outcome with `keel mission log <id> --entry "<text>"`, optionally runs `keel mission digest <id>` for long logs, then reruns board health commands before choosing the next phase.
5. **Do Not Mix Phases In One Worker**: If the work changes from execution to planning or research, stop and hand off to the matching workflow context instead of continuing in the old one. Parent context reads and directly coupled closure steps are fine; silent mission re-scoping is not. Only parallelize workers when their artifacts and ownership do not overlap.

## Decision Resolution Hierarchy

When faced with ambiguity, resolve decisions in this descending order:
1.  **ADRs**: Binding architectural constraints.
2.  **CONSTITUTION**: The philosophy of collaboration.
3.  **ARCHITECTURE**: Source layout and technical boundaries.
4.  **PLANNING**: PRD/SRS/SDD authored for the current mission.

## Foundational Documents

These define the constraints and workflow of this repository:

| Document | Purpose |
|----------|---------|
| `README.md` | Entrypoint and canonical document navigation |
| `INSTRUCTIONS.md` | Step-by-step procedural loops and checklists |
| `CONSTITUTION.md` | Collaboration philosophy and decision hierarchy |
| `ARCHITECTURE.md` | Implementation architecture and flow model |
| `CONFIGURATION.md` | Role-based and config-driven topology |
| `RELEASE.md` | Release process and artifacts |
| `.keel/adrs/` | Binding architecture decisions |

Use this order when interpreting constraints: ADRs → Constitution → Architecture → Configuration → Planning artifacts.

## Project Overview

This repository is `port` — agentic compute orchestration in Firecracker VMs.

| Path | Purpose |
|------|---------|
| `README.md` | Current project description |
| `flake.nix` | Nix flake for the dev shell and shared tooling |
| `justfile` | Repo-local workflow wrappers |
| `AGENTS.md` | Shared agent workflow contract |
| `INSTRUCTIONS.md` | Procedural instructions and checklists |
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
- `keel ...` for all planning, mission, execution, research, and verification workflows.

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
| `just mission [id]` | Convenience wrapper for `keel mission show` with auto-selection when `id` is omitted |
| `just screen [id]` | Convenience wrapper for `keel screen` |

### `keel` Board Workflow Commands

Run `keel --help` for the full command tree. Common commands:

| Category | Commands |
|----------|----------|
| Discovery | `keel bearing new <name>` `keel play <id>` `keel bearing research <id>` `keel bearing assess <id>` `keel bearing list` |
| Planning | `keel epic new "<Title>" --problem "<Problem>"` `keel voyage new "<Title>" --epic <epic-id> --goal "<Specific outcome>"` |
| Execution | `keel story new "<title>" [--type <type>] [--epic <epic-id> [--voyage <voyage-id>]]` |
| Board Ops | `keel mission next [<id>]` `keel next --role manager` `keel flow` `keel doctor` `keel generate` `keel config show` `keel mission show <id>` |
| Routines | `keel routine new "<name>"` `keel routine list` `keel routine show <id>` `keel pulse` |
| Verification | `keel verify run <id>` `keel verify detect` `keel verify recommend` |

## Story and Milestone State Changes

Use CLI commands only. Do not move `.keel` files manually.

| Action | Command |
|--------|---------|
| Start | `keel story start <id>` |
| Reflect | `keel story reflect <id>` |
| Submit | `keel story submit <id>` |
| Reject | `keel story reject <id> "reason"` |
| Accept | `keel story accept <id> --role manager` |
| Ice | `keel story ice <id>` |
| Thaw | `keel story thaw <id>` |
| Voyage plan | `keel voyage plan <id>` |
| Voyage done | `keel voyage done <id>` |
| Bearing assess | `keel bearing assess <id>` |
| Bearing lay | `keel bearing lay <id>` |
| Mission activate | `keel mission activate <id>` |
| Mission achieve | `keel mission achieve <id>` |

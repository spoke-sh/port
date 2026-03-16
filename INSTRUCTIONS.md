# INSTRUCTIONS.md

Procedural instructions and workflow guidance for agents working with the Port repository.

## The Tactical Loop

Port uses Keel as its project management engine. Your job is to perform "tactical moves" that push work through the state machine while eliminating drift.

Every session follows this deterministic cycle:

1.  **Mission Orientation**: Start by running `just keel mission next --status`. This gives you the top 3 high-signal moves required by the engine. Check `just keel flow --scene` to quickly visualize if the workflow is autonomous or blocked waiting for human input.
2.  **Role Selection**: Identify if you are a `manager` (planning/decisions) or an `operator` (implementation). Do not drift across these roles in a single atomic change.
3.  **Execute Move**: Perform exactly ONE move (e.g., plan a voyage, implement a story, fix a diagnostic).
4.  **Seal Move**: Close the loop with `story submit`, `voyage plan`, or `bearing lay`. This mutates the `.keel` state. Ensure the pacemaker is stable (committed heartbeat).
5.  **Log & Commit**:
    - Record your move in the mission `LOG.md`.
    - **Pace-setting**: Execute `just keel poke "Sealing move: <summary>"` to synchronize the pacemaker with the board state.
    - **Commit**: Execute `git commit`. The pre-commit hook will automatically run `just check` and `just test`. Resolve any issues if the commit is rejected.
6.  **Re-orient**: After the commit lands, run `just keel doctor --status` and `just flow` to see what the board needs next.
 This is the "plug the chord back in" moment — you reconnect to the board's current state. If the delivery lane has ready work, start the next loop immediately. Only stop to ask the human when you reach a manual lane (design direction, bearing assessment, or human verification).

## Primary Workflows

### Operator (Implementation)
Focus on **evidence-backed delivery**.
- **Context**: `just keel story show <id>` and `just keel voyage show <id>`.
- **Action**: Implement requirements, record proofs with `just keel story record`, and `submit`.
- **Constraint**: Every AC must have a proof.

### Manager (Planning)
Focus on **strategic alignment and unblocking**.
- **Context**: `just keel epic show <id>` and `just flow`.
- **Action**: Author `PRD.md`, `SRS.md`, `SDD.md`, and decompose stories.
- **Constraint**: Move voyages from `draft` to `planned` only when requirements are coherent.

### Explorer (Research)
Focus on **technical discovery and fog reduction**.
- **Context**: `just keel bearing list`.
- **Action**: Fill `BRIEF.md`, collect `EVIDENCE.md`, and `assess`.
- **Constraint**: Graduate to epics only when research is conclusive.

## Human Interaction & Pokes

Keel's autonomous flow is governed by a physical battery metaphor. If the system is IDLE (LIGHT OFF) due to battery decay, it requires a "spark" to resume autonomy.

If a human user pokes you (e.g., "I'm poking you", "Wake up"), you MUST:
1.  **Energize the System**: Immediately execute `just keel poke "Human interaction in chat"`. This physically mutates the board state and recharges the battery, signaling the engine to resume autonomous flow.
2.  **Autonomous Scan**: Run `just keel mission next --status` and `just keel pulse` to identify any new work that has become ready or materialized.
3. **Visual Confirmation**: Run `just keel flow --scene` to verify the light is now ON and the circuit is closed.

## Autonomous Backlog Discharge

As long as the system is **AUTONOMOUS (LIGHT ON)** and the circuit is healthy (no blown capacitors), you are responsible for discharging the delivery backlog.

1.  **Identify Ready Work**: Scan the delivery lane for stories in `backlog` that are not blocked by dependencies.
2.  **Autonomous Start**: For each ready story, execute `just keel story start <id>`. 
3.  **Rube Goldberg Loop**: Transitioning a story to `in-progress` mutates the board, which physically recharges the system battery, keeping the circuit closed and the engine running. 
4.  **Priority**: Discharging the backlog is your primary tactical objective once energized. You must continue until the backlog is empty or the circuit trips.
5.  **Loop Closure**: After every successful implementation or transition, you MUST stabilize the pacemaker by executing `just keel poke "Sealing move: <summary>"` and committing the `.keel/heartbeat` alongside the board changes. This applies to ALL work, including storyless gardening or engine changes. Do not leave the loop open.

## Global Hygiene Checklist

Apply these checks to **every change** before finalizing work:

1. **Doctor First**: `just keel doctor` is the ultimate source of truth for board integrity. You MUST run the doctor at the start of every session. If the doctor reports errors or "Short Circuits", you MUST prioritize fixing those diagnostic orders before attempting any other work or architectural changes.
2. **The Health Loop**: Use `just keel health --scene` for high-level triage. Subsystems are mapped as follows:
   - **NEURAL**: Stories (ID consistency, AC completion)
   - **MOTOR**: Voyages (Structure, SRS/SDD authorship)
   - **STRATEGIC**: Epics (PRD, Goal lineage)
   - **SENSORY**: Bearings (Research, Evidence quality)
   - **SKELETAL**: ADRs (Architecture decisions)
   - **VITAL**: Missions (Strategic achievement)
   - **AUTONOMIC**: Routines (Cadence, materialization)
   - **CIRCULATORY**: Workflow (Graph integrity, topology)
   - **PACEMAKER**: Heartbeat (System energization and commit stability)
   - **KINETIC**: Delivery (Backlog liquidity, execution capacity)
3. **Pacemaker Protocol**: The system's heartbeat (.keel/heartbeat) is its pacemaker. You MUST ensure the pacemaker is stable (committed) before concluding any unit of work. Every commit MUST be preceded by a `just keel poke "Sealing move: <summary>"` to refresh the system's pulse and reflect the latest change, especially when working without a story. Uncommitted energy is a signal of an open tactical loop and will trigger a CRITICAL status in the Med-Bay bio-scan. Any commit that includes `.keel/heartbeat` MUST append the output of `just keel doctor --status` to the commit message body so reviewers can see the board's importance snapshot at the time of the commit.
4. **Gardening First**: You MUST tend to the garden (fixing `doctor` errors, discharging automated backlog, and resolving structural drift) BEFORE notifying the human operator or requesting input. 
5. **Notification Threshold**: Only request human intervention when you reach a "Manual Lane" that requires design direction or a decision on application behavior (e.g., assessing a Bearing, planning a Voyage, or human verification of a complex Story).
6. **Automated Guardrails**: You no longer need to run `just fmt-check` or `just clippy` manually before every commit. The git pre-commit hook (installed via `just keel hooks install`) automatically enforces these checks. If a commit fails, resolve the reported lints or test failures and try again.
7. **Lifecycle Before Commit**: Run board-mutating lifecycle commands before the atomic commit when they generate or rewrite `.keel` artifacts (for example `story submit`, `voyage plan`, `voyage done`, `bearing assess`, `bearing lay`). After the transition, inspect `git status` and include the resulting `.keel` churn in the same commit.
8. **Atomic Commits**: Commit once per logical unit of work. Use [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat:` (new feature)
   - `fix:` (bug fix)
   - `docs:` (documentation)
   - `refactor:` (code change, no behavior change)
   - `test:` (adding/updating tests)
   - `chore:` (build/tooling)
9. **Mission Loop Discipline**: For mission-driven work, return to the mission steward loop after every completed story, planning unit, or bearing instead of continuing ad hoc from the last worker context.
10. **Knowledge Quality Bar**: Prefer no new knowledge over low-signal knowledge. A new knowledge entry should be novel, reusable across stories, and materially reduce future drift; otherwise link existing knowledge or omit capture entirely.

## Compatibility Policy (Hard Cutover)

At this stage of development, this repository uses a **hard cutover** policy by default.

1. **No Backward Compatibility by Default**: Do not add compatibility aliases, dual-write logic, soft-deprecated schema fields, or fallback parsing for legacy formats unless a story explicitly requires it.
2. **Replace, Don't Bridge**: When introducing a new canonical token, field, command behavior, or document contract, remove the old path in the same change slice.
3. **Fail Fast in Validation**: `keel doctor` and transition gates should treat legacy or unfilled scaffold patterns as hard failures when they violate the new contract.
4. **Single Canonical Path**: Keep one source of truth for rendering, parsing, and validation; avoid parallel implementations meant only to preserve old behavior.
5. **Migration Is Explicit Work**: If existing board artifacts need updates, handle that in a dedicated migration pass/story instead of embedding runtime compatibility logic.

## Commands

### Command execution model (DRY)

Use one path for each concern:

- `nix develop` for the repository shell and shared tooling.
- `just ...` for repo/build/test workflows.
- `just keel ...` for all board/workflow operations.

### `just` workflow commands

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

### `just keel` board workflow commands

Run `just keel --help` for the full command tree. The core commands you should rely on:

| Category | Commands |
|----------|----------|
| Discovery | `just keel bearing new <name>` `just keel bearing research <id>` `just keel bearing assess <id>` `just keel bearing list` |
| Planning | `just keel epic new "<name>" --problem "<problem>"` `just keel voyage new "<name>" --epic <epic-id> --goal "<goal>"` |
| Execution | `just keel story new "<title>" [--type <type>] [--epic <epic-id> [--voyage <voyage-id>]]` |
| Board Ops | `just keel mission next [<id>]` `just keel next --role manager` `just keel next --role operator` `just keel flow` `just keel doctor` `just keel generate` `just keel config show` `just keel mission show <id>` |
| Lifecycle | Story/voyage/epic transitions in the table below |

## Story and Milestone State Changes

Use CLI commands only; do not move `.keel` files manually.

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

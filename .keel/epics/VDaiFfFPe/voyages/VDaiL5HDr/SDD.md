# Mission Verification And Help Simplification - Software Design Description

> Give operators one concise mission verification entrypoint, a simpler just surface, and foundational docs/help that are fast to audit.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage introduces one board-backed verification path and one canonical
documentation structure instead of adding more ad hoc examples. The design
keeps `just` thin: a root `just mission` recipe will orchestrate the existing
mission report script and a repo-local report will summarize mission
state, progress, recent achievements, and high-level human artifacts. In parallel, the `just` file is split
into logical modules so the root help only shows common workflows, and the
root documentation is promoted into canonical contracts for configuration,
architecture, constitution, release, and evaluation.

## Context & Boundaries

### In Scope

- mission verification orchestration and reporting
- `just` module split and top-level help reduction
- root documentation contracts and README map
- top-level CLI help simplification
- removal of cargo-runner examples from user-facing docs

### Out of Scope

- new runtime capabilities
- new release automation
- removal of demo automation itself

```
┌──────────────────────────────────────────────────────────┐
│      Mission Verification And Help Simplification        │
│                                                          │
│  just mission ─────────────────> mission report          │
│       │                             │                    │
│       ├────────> just modules <─────┤                    │
│       └────────> root docs/help ----┘                    │
└──────────────────────────────────────────────────────────┘
           ↑                                   ↑
       keel board                         README / port --help
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| mission/epic/voyage/story board artifacts plus `keel mission next` | internal CLI + board files | canonical board-backed mission report inputs | current Keel CLI |
| `just` modules | toolchain | hide low-signal recipes from root help while keeping them available | `just 1.46.0` |
| `clap` help rendering in `port-cli` | internal | keeps the top-level CLI help concise and testable | current workspace crate |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Mission report implementation | use a repo-local script driven by existing Keel commands instead of adding a new product CLI command | fastest path to a board-backed report without inflating Port's runtime surface |
| Help simplification strategy | keep a small set of common examples in root help and move detailed flows to `CONFIGURATION.md` plus focused docs | preserves discoverability while removing the wall of text |
| `just` organization | split recipes into submodules and keep only common entrypoints at root | reduces default noise while preserving access to specialized workflows |
| Documentation contract | add root-level foundational docs instead of expanding README further | gives one obvious place for durable contracts and reduces duplication |

## Architecture

The voyage touches three layers:

1. board-backed mission verification orchestration
2. developer workflow surface organization through `just`
3. canonical documentation and CLI help surfaces

## Components

### Mission Report Script

- Purpose: select the relevant mission, run the canonical board views, and
  render a concise terminal summary ending with recent achievements and a
  high-level artifact gallery.
- Interface: invoked from `just mission`, optionally with a mission id.
- Behavior: derives mission identity from board files, prints mission status and
  child progress, shows the next step when relevant, and highlights the most
  human-meaningful proof artifacts linked to the mission.

### `just` Modules

- Purpose: separate common root workflows from detailed or demo workflows.
- Interface: root `just` recipes plus module-specific listings such as
  `just --list keel` or `just --list demo`.
- Behavior: keep root help short while preserving access to specialized
  workflows under explicit modules.

### Root Documentation Set

- Purpose: define the durable repository contracts for configuration,
  architecture, constitution, release, and evaluations.
- Interface: root markdown documents linked from README and help text.
- Behavior: centralize detail, especially long-form examples and configuration
  edits, so top-level docs can stay brief.

### CLI Help Surface

- Purpose: present a fast-scanning entrypoint for new operators.
- Interface: `port --help`.
- Behavior: retain the command tree and 2-3 useful examples, then link to
  `CONFIGURATION.md` and focused docs for long-form workflows.

## Interfaces

- `just mission [mission-id]`
- `just --list`
- `just --list <module>`
- `port --help`
- root markdown contracts in the repository root

## Data Flow

1. The maintainer runs `just mission`.
2. `just mission` invokes the mission-report script.
3. The mission-report script reads mission metadata, linked board artifacts,
   and the current `keel mission next` output, then renders a compact summary
   plus recent achievements and a high-level artifact gallery.
4. The maintainer uses root docs and concise help output to drill into detailed
   examples only when needed.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| No active mission exists | mission report selection finds only completed or no missions | fall back to the most recent mission and print that selection explicitly | create or activate a mission before relying on the report for active delivery |
| `just` module split hides an important workflow unintentionally | help proof or manual inspection | add a root entrypoint or clearer module hint | rerun help proof and update README map |
| Detailed examples become harder to find after help reduction | doc audit or README inspection | add direct links from help/README to the canonical root or focused docs | keep detailed examples centralized and linked, not duplicated |
| Cargo-runner examples remain in user-facing docs | search proof | fail the story and replace the stale examples | rerun grep and doc audit before submit |

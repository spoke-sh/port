# Remove Nix Bias From Help Surface - Software Design Description

> Make the help/examples describe generic runtime prerequisites instead of prescribing nix develop.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage is a presentation-layer correction. It removes nix-specific wording
from the help/example surfaces added in the previous follow-up and replaces it
with generic prerequisite language:

- example commands still assume repo-relative config paths where relevant;
- local launch examples now talk about required tools and `port doctor` instead
  of prescribing a Nix shell; and
- supporting docs mirror the same runtime-agnostic guidance.

## Context & Boundaries

### In Scope

- `port --help` wording in `port-cli`;
- README and operator docs that were changed by the help-example clarification;
- verification proving absence of the nix-specific prescription.

### Out of Scope

- development-environment docs outside the help/example correction;
- runtime behavior; and
- automatic installation of dependencies.

```
┌─────────────────────────────────────────────────────────────┐
│                    Help / Docs Surface                     │
│  prerequisite wording -> tool availability + port doctor   │
└─────────────────────────────────────────────────────────────┘
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `port doctor` | CLI/runtime | Canonical prerequisite gate for local launch | current workspace API |
| README / operators docs | Documentation | Must stay aligned with `port --help` | checked-in docs |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Prerequisite wording | Refer to required tools on `PATH` and `port doctor`, not `nix develop` | Port is not tied to Nix as a runtime contract |
| Verification | Check for the presence of generic prerequisite wording and the absence of `nix develop` in the touched help/example surfaces | The regression is specifically wording bias |

## Architecture

This voyage changes only the CLI help string and the aligned docs.

## Components

- CLI help:
  describes the runtime assumptions in generic terms.
- README / operator docs:
  repeat the same tool-availability and `port doctor` guidance.
- Evidence:
  proves the wording is corrected on the canonical CLI surface.

## Interfaces

- `port --help`
- `README.md`
- `docs/operators.md`

## Data Flow

1. Operator reads `port --help`.
2. Help text tells them they need the required tools available and should use
   `port doctor` as the launch gate.
3. README and operator docs reinforce the same expectation without treating Nix
   as a runtime requirement.

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Help still implies Nix is required to run Port | Manual review or CLI proof | Replace nix-specific wording with generic prerequisite language | Keep runtime assumptions framed in terms of tools and validation |
| Docs drift from help again | Review / grep proof | Align docs in the same change slice | Update help and docs together |

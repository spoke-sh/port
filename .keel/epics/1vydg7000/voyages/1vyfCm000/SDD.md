# Clarify Help Examples - Software Design Description

> Make port --help examples explicit about their environment prerequisites and runnable workflow order.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage fixes a product-surface mismatch rather than changing runtime
behavior. The design is to tighten three surfaces together:

- `port --help` gains explicit prerequisite language and a short runnable
  sequence instead of isolated commands that look environment-independent.
- README/operator docs mirror that guidance so the same expectation appears in
  the CLI and the checked-in docs.
- Verification runs the updated help-example flow directly with the stated
  environment assumptions.

## Context & Boundaries

In scope:

- help text and examples in `port-cli`;
- README / operator docs that explain the prerequisite boundary; and
- verification evidence for the updated example workflow.

Out of scope:

- installing dependencies automatically;
- changing how `port doctor` or `port machine launch` behave at runtime; and
- adding new runtime capabilities.

```
┌──────────────────────────────────────────────────────────────┐
│                        Port CLI Help                        │
│   examples + prerequisite note + workflow ordering          │
└──────────────────────────────────────────────────────────────┘
                 │                              │
                 ▼                              ▼
          README / operators docs         verification proof
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `nix develop` shell | Environment | Supplies Firecracker and artifact-build dependencies used by the example workflow | current flake |
| `port doctor` | CLI/runtime | Canonical preflight gate that explains why local launch may fail | current workspace API |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Example framing | Show a short ordered workflow plus explicit prerequisite note | The user issue is that standalone examples looked directly runnable when they were not |
| Verification style | Prove the flow with the built `port` binary in the documented environment | Keeps evidence on the real product surface |

## Architecture

The voyage updates one presentation layer across two outputs:

- CLI help in `crates/port-cli`;
- checked-in docs in `README.md` and `docs/operators.md`.

## Components

- Help text:
  clarifies that the local artifact/launch examples assume `nix develop` (or an
  equivalent environment) and that `port doctor` gates local launch.
- Docs:
  restate the same prerequisite boundary in longer operator-facing language.
- Verification:
  runs the updated example sequence so evidence matches what the help actually
  publishes.

## Interfaces

- `port --help`
- `README.md`
- `docs/operators.md`

## Data Flow

1. Operator reads `port --help`.
2. Help text makes the environment requirement explicit before local artifact or
   launch examples are attempted.
3. Operator follows the ordered example flow in the documented environment.
4. If prerequisites are still missing, `port doctor` is the explicit gate that
   explains why launch is unavailable.

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Help text still looks like standalone local launch should work anywhere `port` runs | Manual review or direct CLI repro | Update the help wording and example order | Keep prerequisite language adjacent to the examples |
| Docs drift from `port --help` again | Doc review / grep proof | Fail the story review and align the wording | Update CLI and docs in the same slice |

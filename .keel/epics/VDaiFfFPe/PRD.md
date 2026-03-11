# Operator Signal And Documentation Foundations - Product Requirements

> Give Port maintainers one concise way to judge mission progress and one
> coherent documentation contract for operating, configuring, and releasing the
> product without paging through duplicated examples.

## Problem Statement

Port exposes real delivery progress, but the signal is hard to audit quickly:

- `just` shows an overwhelming flat list with demo tasks mixed into common
  workflows
- there is no single `just mission` command that presents compact mission
  status and a high-level artifact gallery
- `port --help` dumps a wall of examples that is too large to scan during
  review
- foundational repository docs such as configuration, release, architecture,
  and evaluation contracts are missing or scattered across deep doc pages
- user-facing docs still contain stale `cargo run -p port-cli` examples instead
  of the canonical `port` command

That leaves maintainers without a fast success surface for the current mission
and makes documentation review noisier than the underlying product state.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Give maintainers one concise mission verification surface. | `just mission` shows a compact board-backed report and a high-level artifact gallery | First voyage complete |
| GOAL-02 | Make the repo's primary docs auditable from the root. | Root documentation covers constitution, architecture, configuration, release, and evaluation expectations | First voyage complete |
| GOAL-03 | Reduce help-surface noise without losing discoverability. | Root `just` help and `port --help` keep only common workflows and 2-3 high-value examples | First voyage complete |
| GOAL-04 | Publish one canonical operator vocabulary. | User-facing examples use `port` and point detailed flows to `CONFIGURATION.md` and focused docs | First voyage complete |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Port Maintainer | Reviews active work, verifies progress, and needs a fast status read before handoff or release. | One concise verification path that ends with clear board signal |
| Operator / Evaluator | Needs to understand how to run Port and where detailed workflows live. | A small set of obvious entry documents and concise examples |
| Contributor | Updates runtime or docs surfaces and needs a stable documentation contract. | Clear root-level guidance on configuration, architecture, evaluation, and release expectations |

## Scope

### In Scope

- [SCOPE-01] A `just mission` entrypoint and supporting report surface for mission verification.
- [SCOPE-02] Reorganization of the `just` surface into logical modules with a concise default help path.
- [SCOPE-03] Root-level foundational documents for Port's constitution, architecture, configuration, release, and evaluations.
- [SCOPE-04] Simplified `port --help`, README, and operator docs with detailed examples centralized into `CONFIGURATION.md`.
- [SCOPE-05] Removal of stale cargo-runner examples from user-facing docs and help.

### Out of Scope

- [SCOPE-90] New runtime, artifact, or hosted-product capabilities beyond documentation and verification surfaces.
- [SCOPE-91] A full release automation pipeline or packaging system redesign.
- [SCOPE-92] Removal of demo automation itself; demo tasks may remain available outside the default help surface.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Port must provide a single `just mission` mission report path that presents a concise board-backed report plus a high-level artifact gallery. | GOAL-01, GOAL-03 | must | Maintainers need one command to judge progress without reading multiple tools by hand. |
| FR-02 | The repository must reorganize `just` into logical modules and keep the default top-level help focused on common workflows. | GOAL-01, GOAL-03 | must | The current flat help surface obscures the tasks people actually use. |
| FR-03 | Port must publish root-level foundational docs covering constitution, architecture, configuration, release, and evaluation expectations. | GOAL-02, GOAL-04 | must | The current documentation contract is fragmented and hard to audit from the repo root. |
| FR-04 | Port must simplify the top-level CLI and README examples to a small set of useful entry examples and move detailed configuration workflows into `CONFIGURATION.md`. | GOAL-02, GOAL-03, GOAL-04 | must | Help output should guide operators quickly without burying the real signals. |
| FR-05 | User-facing docs and help must use the canonical `port` command surface instead of `cargo run -p port-cli`. | GOAL-04 | must | The published operator vocabulary should match the product identity. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Mission verification output must be derived from board truth and not require hand-maintained summary text. | GOAL-01, GOAL-03 | must | The signal surface fails if it drifts from the underlying board. |
| NFR-02 | Documentation simplification must reduce duplication by linking to canonical root docs instead of copying long example blocks into multiple places. | GOAL-02, GOAL-03, GOAL-04 | must | Smaller help surfaces stay maintainable only if detail lives in one canonical place. |
| NFR-03 | The first voyage must finish with board-linked implementation stories and concrete proof commands rather than another documentation-only placeholder. | GOAL-01, GOAL-02, GOAL-03, GOAL-04 | should | The user asked for an executable success surface, not a passive backlog addition. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Mission verification path | command proof + inspection | recorded `just mission` output and board-backed report |
| `just` help simplification | command proof | `just` default output plus module listings |
| Root documentation | inspection + doc audit | root docs and README link review |
| CLI help simplification | automated help test + command proof | updated `port --help` output and tests |
| Cargo-runner removal | search proof | repository-wide doc grep for stale `cargo run -p port-cli` references |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Maintainers care more about fast status signal than exhaustive inline examples. | A simplified help surface could feel too terse. | Validate by keeping canonical detail in root docs and focused pages. |
| Existing mission, epic, voyage, and story board artifacts are stable enough to power a mission report without adding a new product CLI command. | The report may need deeper product integration later. | Validate in the first verification story. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Is the current artifact gallery enough, or will Port eventually need a richer product-level demo/media view? | Maintainer | Open |
| Which long-form examples should remain in focused docs versus move entirely into `CONFIGURATION.md`? | Maintainer | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Maintainers can run one `just mission` command and immediately see mission status, child progress, next step, recent achievements, and high-level artifacts.
- [ ] Root docs explain Port's constitution, architecture, configuration, release, and evaluation expectations without requiring a deep docs crawl.
- [ ] `port --help` and the README keep only a short set of useful examples and point detailed workflows to `CONFIGURATION.md` and focused docs.
- [ ] User-facing docs no longer publish `cargo run -p port-cli` as the operator path.
<!-- END SUCCESS_CRITERIA -->

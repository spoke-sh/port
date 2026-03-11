# Mission Verification And Help Simplification - Software Requirements Specification

> Give operators one concise mission verification entrypoint, a simpler just surface, and foundational docs/help that are fast to audit.

**Epic:** [VDaiFfFPe](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] A `just mission` verification path and supporting mission-status report.
- [SCOPE-02] Logical `just` modules with a concise default help surface.
- [SCOPE-03] Root documentation for configuration, constitution, architecture, release, and evaluations.
- [SCOPE-04] Simplified top-level CLI help and README examples with detailed workflows centralized into `CONFIGURATION.md`.
- [SCOPE-05] Replacement of stale cargo-runner examples in user-facing docs and help.

### Out of Scope

- [SCOPE-90] New runtime features or new hosted/service behavior.
- [SCOPE-91] Release automation or binary distribution changes beyond documenting the current release contract.
- [SCOPE-92] Removal of demo recipes themselves; only their visibility in the default help surface changes.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Mission charters, epic/voyage/story board artifacts, and `keel mission next` provide enough stable signal to drive a repo-local mission report. | dependency | `just mission` would need a custom parser or a new product-side command |
| `just` module support is available in the repository toolchain. | dependency | Help simplification would require a weaker flat-file grouping only |
| Port can keep a concise help surface without losing discoverability if detailed flows are linked from canonical docs. | assumption | Users may lose important workflows unless links are clear and well-placed |

## Constraints

- Use `port` as the only published user-facing command surface in docs and help.
- Keep root `just` help centered on common workflows; deeper and demo workflows should remain discoverable through modules, not the default list.
- Do not add compatibility aliases or dual documentation paths for old help contracts.
- Keep the board doctor-clean after structural changes.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `just mission` must present the mission report and finish with a compact summary that shows mission status, child progress, next step, recent achievements, and a human-facing artifact gallery. | SCOPE-01 | FR-01 | command proof + inspection |
| SRS-02 | The repository must split `just` into logical modules and keep the default root help focused on common workflows while keeping demo tasks available outside the default list. | SCOPE-02 | FR-02 | command proof |
| SRS-03 | Port must publish root-level `CONSTITUTION.md`, `ARCHITECTURE.md`, `CONFIGURATION.md`, `RELEASE.md`, and `EVALUATIONS.md` documents that reflect the real current product contract. | SCOPE-03 | FR-03 | inspection + doc audit |
| SRS-04 | `port --help` and the README must keep only 2-3 useful examples and point detailed workflow examples and config edits to `CONFIGURATION.md` and focused docs. | SCOPE-04 | FR-04 | automated help test + command proof + inspection |
| SRS-05 | User-facing docs and help must replace `cargo run -p port-cli` examples with the canonical `port` command surface. | SCOPE-05 | FR-05 | search proof + inspection |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The mission report must derive status from board truth rather than manually duplicated mission summaries. | SCOPE-01, SCOPE-02 | NFR-01 | inspection + command proof |
| SRS-NFR-02 | Documentation simplification must reduce duplication by making one canonical root location responsible for detailed examples and contracts. | SCOPE-03, SCOPE-04, SCOPE-05 | NFR-02 | inspection + doc audit |
| SRS-NFR-03 | The voyage must land executable proof commands for the new help and mission surfaces rather than relying on narrative-only review. | SCOPE-01, SCOPE-02, SCOPE-04, SCOPE-05 | NFR-03 | command proof + automated test |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

# Remove Nix Bias From Help Surface - Software Requirements Specification

> Make the help/examples describe generic runtime prerequisites instead of prescribing nix develop.

**Epic:** [1vydg7000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

- [SCOPE-01] Remove nix-specific wording from the canonical help/examples and
  align supporting docs on generic runtime prerequisites.
- [SCOPE-02] Record CLI-level evidence that the help surface now describes tool
  availability and `port doctor` gating without prescribing Nix.
- Out of scope: changing Port runtime behavior, adding installers, or removing
  repository development-shell documentation from unrelated sections.

## Assumptions & Dependencies

<!-- What we assume to be true; external systems, services, or conditions we depend on -->

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| The real issue is product-surface wording, not runtime dependence on Nix. | Assumption | The voyage would need to widen into a runtime implementation change. |
| `port doctor` remains the canonical way to surface missing prerequisites in a runtime-agnostic way. | Dependency | Another CLI surface would be needed to explain launch failures. |

## Constraints

- The fix must remove the prescriptive `nix develop` language from the help and
  example workflow surfaces added in the previous follow-up.
- The guidance still has to explain why local launch can fail when required
  tools are absent.
- Verification must prove both presence of the new generic prerequisite wording
  and absence of the nix-specific prescription in the relevant surfaces.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `port --help` must describe local example prerequisites in runtime-agnostic terms such as required tools on `PATH` and `port doctor` gating, without prescribing `nix develop`. | SCOPE-01 | FR-01 | automated test + CLI proof |
| SRS-02 | README and operator docs must align with the help text by describing generic prerequisite availability instead of treating Nix as the canonical runtime path for Port. | SCOPE-01 | FR-05 | manual review + doc proof |
| SRS-03 | Recorded evidence must show the updated help surface omits the nix-specific prescription and still directs operators to `port doctor` and the required tool availability. | SCOPE-02 | FR-01 | CLI proof |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The help and docs must remain honest about prerequisites without implying that Nix is required to run Port. | SCOPE-01, SCOPE-02 | NFR-01 | inspection + CLI proof |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

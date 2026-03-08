# Mac Operator Shell Compatibility - Software Requirements Specification

> Keep the Port development shell usable on macOS by removing Linux-only package assumptions while preserving Linux launch tooling on Linux hosts.

**Epic:** [1vz3ck000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

- [SCOPE-01] Make `nix develop` evaluate on macOS without requiring unsupported
  Linux-only packages.
- [SCOPE-02] Preserve the Linux development shell inputs needed for local
  Firecracker launch and artifact workflows on Linux hosts.
- [SCOPE-03] Document the macOS shell boundary so operators understand which
  runtime tools stay Linux-only.
- Out of scope: AVF runtime implementation, macOS local Firecracker launch,
  and any change to the canonical Linux execution requirements.

## Assumptions & Dependencies

<!-- What we assume to be true; external systems, services, or conditions we depend on -->

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| Nix can evaluate a Darwin dev shell from this Linux host. | Dependency | Verification would require a real macOS host instead of cross-platform eval. |
| Firecracker, `iproute2`, and `iptables` remain Linux-only runtime tools in Port's current product shape. | Constraint | The shell split would need a different packaging model. |

## Constraints

- Keep one `flake.nix`; do not fork separate Linux and macOS flakes.
- Do not remove Linux launch tooling from Linux hosts just to make Darwin
  evaluation pass.
- Make the macOS shell honest about current capability: repo tooling is
  available, Linux runtime tools are not.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The default development shell must evaluate on macOS without attempting to realize unsupported Linux-only packages. | SCOPE-01, SCOPE-03 | FR-04 | Darwin flake eval + shell inspection |
| SRS-02 | The Linux development shell must continue to include the current Firecracker and Linux networking tools required by Port's local launch workflow. | SCOPE-02 | FR-01 | Linux shell eval + flake inspection |
| SRS-03 | Operator-facing docs or shell messaging must explain that the macOS shell omits Linux-only runtime tools while Port's Linux launch lane remains unchanged. | SCOPE-03 | NFR-02 | doc review + shell inspection |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The shell fix must not require impurity flags or unsupported-system overrides to work on macOS. | SCOPE-01 | NFR-01 | Darwin flake eval |
| SRS-NFR-02 | The voyage must preserve one shared operator shell contract, with platform-specific package selection expressed explicitly inside the flake. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-02 | flake review + doc review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

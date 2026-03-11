# PVM Runtime Admission And Placement - Software Design Description

> Turn x86_64 Firecracker PVM from a documented foundation into a runtime
selection and placement contract across local and hosted Port surfaces

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage treats PVM as a real execution lane inside Port's existing machine
and hosted architecture rather than as a documentation-only option. The design
adds one shared capability vocabulary, teaches the Firecracker runtime to use
that vocabulary for PVM launch selection, and extends the hosted control path
to admit or deny PVM machines based on node capability instead of implicit
assumptions.

## Context & Boundaries

### In Scope

- `port-model` capability and inventory contracts for x86_64 PVM readiness
- `port-runtime` local launch selection and hosted admission checks
- hosted protocol and SDK surface additions needed to expose node/runtime PVM
  capability
- canonical CLI/help/docs updates for the new runtime boundary

### Out of Scope

- a production scheduler
- an `aarch64` PVM implementation
- replacing the current guest protocol
- pretending an unprepared host can successfully run a PVM guest

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `port-model` | workspace crate | Canonical config and machine/host/node contracts | workspace |
| `port-runtime` | workspace crate | Launch, status, doctor, hosted control-plane, and node-agent behavior | workspace |
| `port-hosted-protocol` and `port-sdk` | workspace crates | Shared HTTP payloads for hosted inventory and capability routing | workspace |
| `firecracker-pvm` host kit | external operator dependency | Required binary/kernel lane for true PVM launch on prepared hosts | operator-provided |
| `keel` verification flow | tooling | Evidence, traceability, and transition enforcement | repo-local |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Capability vocabulary | Represent PVM readiness explicitly instead of inferring it only from `ProtectionMode` or provider names. | Launch and placement need a shared truth that works locally and in hosted mode. |
| Local runtime selection | Keep PVM inside the Firecracker driver and select PVM-specific inputs only after host-kit preflight passes. | PVM is a sub-lane of Firecracker, not a second CLI or second driver family. |
| Hosted admission | Gate hosted PVM machines on node capability before guest operations or launch attempts proceed. | Port must avoid false promises when a node cannot satisfy the PVM contract. |
| Verification posture | Prove capability selection and admission on normal developer hosts; reserve true PVM boot proof for prepared hosts. | Most environments will not have the custom host kit, but the product still needs trustworthy evidence. |

## Architecture

The voyage adds one vertical slice across four layers:

1. `port-model` gains explicit node or host capability data for the x86_64 PVM
   lane and the sample config seeds that data for local and hosted examples.
2. `port-runtime` resolves a PVM eligibility decision before launch or hosted
   placement. Local launch uses it to select the PVM VMM path; hosted flows use
   it to admit or deny machine operations.
3. Hosted transport surfaces capability data through shared hosted protocol
   structs and control-plane/node-agent handlers so `port machine ...` can see
   why a hosted PVM machine is or is not placeable.
4. CLI help and operator docs explain the resulting local and hosted workflow in
   one canonical command family.

## Components

- `port-model`
  - Add canonical PVM capability contracts for nodes and/or Firecracker hosts.
  - Validate that `x86_64` remains the only planned PVM runtime lane and
    `aarch64` remains research-only.
- `port-runtime`
  - Add capability resolution helpers used by local launch and hosted
    machine-routing paths.
  - Select the PVM-specific Firecracker binary and emit precise admission
    failures when the host kit or hosted node capability is absent.
- `port-hosted-protocol` / `port-sdk`
  - Add any typed hosted payloads needed to surface node capability or
    placement-admission detail without inventing a separate PVM API.
- Documentation and CLI surface
  - Extend help text, README, `docs/pvm.md`, and the example config to show both
    local and hosted PVM readiness behavior.

## Interfaces

- Config interface:
  - sample config gains explicit hosted-node PVM readiness signals beside the
    existing local Firecracker lane contract
- Runtime interface:
  - local launch path resolves PVM binary/config selection from canonical model
    data
  - hosted machine operations resolve node PVM readiness before launch/status
    claims are made
- Hosted interface:
  - shared hosted payloads expose node capability or placement-admission detail
    through the existing control-plane and node-agent serve paths
- CLI interface:
  - existing `port doctor`, `port machine launch|status`, and help text remain
    canonical; no PVM-only subcommands are added

## Data Flow

1. Operator selects a machine whose `protection_mode` is `pvm`.
2. Port resolves the machine's control contract and substrate driver.
3. Capability resolution checks the requested architecture and protection mode
   against local host-kit readiness or hosted node capability.
4. If capability is satisfied, the runtime selects PVM-specific launch inputs;
   if not, it returns an explicit admission error.
5. CLI/docs/help surface the same decision model so operators can understand the
   failure before retrying with a prepared host or different machine.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Machine requests PVM on unsupported architecture | Model validation or runtime capability resolution | Return explicit unsupported-lane error | Choose supported machine or keep architecture research-only |
| Local host lacks `pti=off` or patched PVM binary | Doctor preflight and local launch eligibility check | Fail launch with host-kit-specific guidance | Prepare the host kit, rerun `port doctor`, retry |
| Hosted node does not advertise x86_64 PVM readiness | Hosted control-plane admission check | Return placement denial with node/capability detail | Route to a prepared node or use a standard machine |
| Standard Firecracker lane regresses while adding PVM capability | Unit tests and workflow proofs | Block story submission | Fix regression before merge |

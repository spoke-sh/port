# AWS PVM Host Kit Preparation - Software Design Description

> Define and implement the AWS-specific prepared-host contract so a normal x86_64 AWS Linux node can advertise cloud-aws PVM readiness through Port-managed preparation and imported inventory surfaces.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage turns the existing prepared-node PVM demo into an explicit AWS host
preparation contract. The design should keep the canonical Port control-plane
preparation path, but bind it to `cloud-aws` placement expectations instead of
leaving the lane as a generic prepared-node proof.

## Context & Boundaries

### In Scope

- model and runtime contract changes that define what an AWS hosted PVM-ready
  node must prove
- preparation and imported-readiness handling for that contract
- provider-aware readiness and prerequisite surfacing

### Out of Scope

- live hosted launch/status/stop proof
- non-AWS providers or arm64 enablement
- external infrastructure provisioning

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐ │
│  │         │  │         │  │         │ │
│  └─────────┘  └─────────┘  └─────────┘ │
└─────────────────────────────────────────┘
        ↑               ↑
   [External]      [External]
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Hosted control-plane inventory | Internal runtime contract | Stores or imports prepared-node readiness used for placement | Current Port hosted API |
| AWS prepared node | External environment | Supplies the custom kernel, patched VMM, and PVM artifact availability this voyage models | x86_64 AWS Linux |
| PVM artifact kit | Runtime input | Must remain distinct from the standard Firecracker artifact lane | Current Port artifact contract |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Provider lane identity | Keep `cloud-aws` as the canonical target instead of `cloud-generic` | The mission is about sealing the real AWS provider contract, not a generic demo alias. |
| Prepared-host ownership | Keep the canonical `prepare-pvm-node` flow and enrich it with AWS-specific readiness semantics | This preserves the Port command model while removing the manual overlay gap. |
| Failure posture | Reject missing AWS prerequisites without fallback to the standard lane | Silent substitution would hide the contract gap the mission exists to close. |

## Architecture

The main design seam is between shared model contract, hosted inventory
readiness, and operator-facing diagnostics:

- model/config contract describes what an AWS hosted PVM node must satisfy
- `prepare-pvm-node` writes or refreshes the imported hosted readiness record
- doctor/status/import surfaces expose whether the AWS hosted PVM lane is ready,
  planned, stale, or malformed

## Components

- Port model and config surfaces: encode the provider-specific host-kit and
  artifact expectations for `cloud-aws`.
- Hosted control-plane preparation path: accepts canonical preparation inputs
  and records a ready AWS PVM lane for placement.
- Diagnostics and readiness renderers: make missing kernel, VMM, artifact, or
  stale-import conditions visible to operators.

## Interfaces

- `port control-plane prepare-pvm-node`
- hosted imported inventory / readiness projection
- `port doctor` and machine/status-facing readiness summaries

## Data Flow

1. Operator targets an x86_64 AWS Linux node with `prepare-pvm-node`.
2. Port validates or records the AWS host-kit facts needed for PVM readiness.
3. Hosted inventory is updated from `planned` to `ready` for the AWS PVM lane.
4. Doctor/status/imported-readiness surfaces expose the new provider-specific
   readiness state or explain why the node is still not ready.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Wrong host kernel or boot args | Preparation validation or doctor check | Fail with explicit AWS hosted PVM guidance | Boot the prepared AWS node into the expected kernel with `pti=off` |
| Missing patched `firecracker-pvm` | Preparation validation or runtime readiness check | Keep AWS PVM lane unready and explain the missing VMM | Install or point Port at the patched binary |
| Missing PVM artifacts or stale imported readiness | Inventory import or doctor/status check | Report stale or incomplete AWS hosted PVM readiness without fallback | Refresh preparation after fixing the artifact inputs |

# Cloud Aws PVM Runtime Proof - Software Design Description

> Route cloud-aws through the prepared AWS PVM lane and prove canonical launch, status, and stop against a live hosted control-plane and node-agent path with provider-aware failure behavior.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage binds the existing hosted control-plane and node-agent path to the
AWS-specific prepared PVM lane. Once the AWS node is ready, the canonical
`cloud-aws` machine lifecycle should succeed through the hosted runtime path;
when the lane is not ready, failure output should stay explicitly AWS- and
PVM-aware.

## Context & Boundaries

### In Scope

- hosted routing for `cloud-aws` PVM lifecycle commands
- provider-aware failure behavior for missing AWS prepared-node prerequisites
- the live operator proof surface for the AWS hosted PVM lane

### Out of Scope

- non-AWS provider rollout
- generalized scheduler work
- arm64 hosted PVM support

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
| Hosted control plane | Internal service | Accepts client lifecycle commands and places `cloud-aws` onto a node | Current Port hosted API |
| Hosted node agent | Internal service | Owns the prepared AWS runtime root and launches or inspects the PVM machine | Current Port hosted node API |
| AWS prepared-node readiness | Input contract | Determines whether `cloud-aws` can use the PVM lane | Output from voyage `VFgclbAzD` |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Runtime identity | Keep `cloud-aws` as the proof surface | The mission exists to seal the real AWS lane rather than a generic placeholder. |
| Failure posture | Reject missing PVM readiness without fallback | Operators need honest failure output that reflects the real provider gap. |
| Proof path | Use the same canonical `port` command family for prepare, launch, status, and stop | The hosted product contract should not introduce an AWS-only verb family. |

## Architecture

The routed path is:

- CLI or operator calls canonical machine lifecycle commands for `cloud-aws`
- hosted control plane resolves placement and readiness against the AWS node
- node agent launches or inspects the PVM runtime on the prepared host
- status and failure responses carry route context and AWS-specific guidance

## Components

- Hosted machine router: selects the AWS prepared PVM lane and rejects fallback.
- Node runtime owner: launches, inspects, and stops the PVM machine on the
  prepared AWS host.
- Operator proof surface: documents and records the end-to-end hosted workflow.

## Interfaces

- `port machine launch --machine cloud-aws`
- `port machine status --machine cloud-aws`
- `port machine stop --machine cloud-aws`
- hosted control-plane and node-agent machine endpoints
- proof docs or artifacts that exercise the same commands

## Data Flow

1. Operator prepares the AWS node through the canonical preparation path.
2. Operator runs `machine launch` for `cloud-aws`.
3. Control plane resolves `cloud-aws` placement against the prepared AWS node.
4. Node agent launches the PVM machine and returns hosted runtime state.
5. Status and stop repeat the same route and preserve AWS-specific context.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| AWS node not prepared for PVM | Placement or launch-time readiness check | Reject `cloud-aws` with actionable PVM preparation guidance | Run the canonical preparation workflow and refresh readiness |
| Hosted route resolves to standard-only lane | Routing guard | Refuse fallback and explain the provider/lane mismatch | Update the node contract or target machine only after AWS PVM readiness exists |
| Live proof environment unavailable | Manual proof setup | Preserve automated coverage and document the blocked live proof leg | Re-run the proof on a prepared x86_64 AWS node once available |

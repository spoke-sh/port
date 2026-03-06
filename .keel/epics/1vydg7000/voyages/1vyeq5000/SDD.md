# Cloud Linux Control Lane - Software Design Description

> Design and partially implement the remote Linux cloud lane, document provider boundaries, and encode the PVM drop decision.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage adds a provider-aware cloud Linux layer on top of the existing host
model and CLI without pretending that full remote launch already exists. The
design has three pieces:

- extend the host model with explicit cloud-provider identity;
- teach `port doctor` and `port machine launch` to report provider-aware remote
  support and actionable "not implemented yet" guidance; and
- publish the cloud/operator support matrix plus the explicit PVM drop decision
  in checked-in docs.

## Context & Boundaries

```
┌───────────────────────────────────────────────────────────────┐
│                           Port CLI                            │
│                                                               │
│  clap/model -> host provider resolution -> doctor/launch UX   │
└───────────────────────────────────────────────────────────────┘
                 │                         │
                 │ config/docs             │ research-backed support matrix
                 ▼                         ▼
        remote Linux host profiles    AWS / GCP / Azure / PVM decision
```

In scope:

- provider-aware host modeling for remote Linux targets;
- CLI diagnostics and launch guardrails for partial cloud support; and
- documentation that explains the supported remote workflow from Linux, macOS,
  and Windows workstations.

Out of scope:

- live cloud launch orchestration;
- SSH command execution against real hosts; and
- any kept PVM lane. This voyage encodes the research-backed decision to drop
  PVM from MVP.

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Existing `HostConnection::Ssh` model | Internal | Remote Linux transport placeholder for partial cloud work | current workspace API |
| Cloud viability bearing `1vydg7000` | Planning | Provider support matrix and PVM recommendation | current board artifact |
| README and operator docs | Documentation | Canonical operator workflow surface | checked-in docs |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Provider identity | Add an explicit provider enum to the host model | Lets the CLI and docs differentiate generic SSH Linux, AWS, GCP, and Azure |
| Partial implementation boundary | Implement provider-aware diagnostics and launch guardrails, not remote launch execution | Satisfies the MVP requirement without overpromising unsupported runtime behavior |
| Azure handling | Mark Azure unsupported for Firecracker MVP | Matches the existing research recommendation |
| PVM lane | Drop from MVP and document the decision | Current research does not justify keeping it in scope |

## Architecture

The cloud lane touches four surfaces:

- `port-model`: host provider identity and sample remote-cloud configs;
- `port-runtime`: provider-aware doctor notes and remote-launch failures;
- `port-cli`: help text/examples that describe the remote Linux boundary; and
- `README` / `docs/*`: operator workflows, provider matrix, and PVM decision.

## Components

- Host model:
  adds provider identity to `HostSpec` so configs can encode generic remote
  Linux, AWS, GCP, or Azure targets.
- Doctor/reporting:
  reads configured remote hosts and emits actionable notes about supported,
  partial, and unsupported provider states.
- Launch guardrail:
  rejects remote cloud targets with provider-specific guidance instead of a
  generic "not local" error.
- Cloud docs:
  publish the support matrix, remote Linux workflow, and explicit PVM drop.

## Interfaces

- Model interface:
  `HostSpec.provider = local | generic-linux | aws | gcp | azure`
- CLI surfaces:
  `port --help`
  `port doctor [--config ...]`
  `port machine launch --machine <name>`
- Docs:
  `README.md`
  `docs/operators.md`
  `docs/cloud.md`

## Data Flow

1. Operator loads a config that includes a remote Linux host and provider.
2. `port doctor` evaluates host/provider intent and prints the support matrix in
   provider-aware notes.
3. `port machine launch` checks the machine host:
   local Linux proceeds as before; remote providers fail fast with explicit next
   steps and current limitations.
4. README and supporting docs explain which environments can run the canonical
   commands directly and which must move execution onto a Linux host.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Operator config omits provider identity for a remote cloud example | Model validation/test failure or docs drift review | Fail the test or update example/docs | Make provider choice explicit |
| Operator targets Azure for Firecracker launch | Provider-aware launch guard | Return unsupported-provider guidance | Move workload to Linux on AWS/GCP or a generic Linux host |
| Operator expects remote launch to already work | Remote host launch request | Return a "not implemented yet" error with current support boundary | Run the local workflow on Linux or wait for later cloud implementation |
| PVM lane ambiguity reappears in docs or config | Review against bearing and docs | Fail review and restate drop decision | Keep PVM out of MVP until new research justifies it |

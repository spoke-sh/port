# Wedge Detector And Cluster Status Fields - Software Design Description

> Introduce a configurable wedge detector that consumes both refresh_age_seconds (node-side) and guest_refresh_age_seconds (guest-side) and surfaces wedged_since, wedge_class, recovery_attempts, last_recovery_action, and recovery_state in port cluster status --format json. No recovery actions yet.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage closes the gap between "Port knows a heartbeat is stale" and "Port publishes a wedge fact on the status contract". It adds no recovery action — the goal is purely to pick a class (node-side vs. guest-side), stamp a first-observed timestamp, and surface both fields on the existing status surface. Downstream consumers (the infra `cluster-wedge` probe, and later the recovery ladder in epic VGzxMc4G4) can then act on a single coherent `wedged_since` / `wedge_class` pair rather than deriving wedge state from raw ages.

The detector runs on the control plane, not the node-agent, for two reasons: (1) both trigger signals (`refresh_age_seconds` from node heartbeats and `guest_refresh_age_seconds` proxied via the node-agent's status response) converge at the control plane, and (2) a silent node-agent cannot be trusted to publish wedge facts about itself.

## Context & Boundaries

<!-- What's in scope, what's out of scope, external actors/systems we interact with -->

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

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|

## Architecture

Two new surfaces and one new background task:

1. **Config surface (`port-model`)** — a new `ClusterDetectionConfig` struct parses `[clusters.<name>.detection]` from `port.toml`, with conservative defaults (e.g. `guest_trigger = 90s`, `node_trigger = 120s`) applied when the block is absent. Validation rejects obviously invalid values.
2. **Detector task (`port-runtime::hosted_control_plane`)** — a periodic task (interval derived from the shorter of the two thresholds) walks the registered-machine list, reads both heartbeat ages via the existing status aggregator, and mutates an in-memory `wedge_state: RwLock<BTreeMap<String, WedgeFact>>` on the control-plane state. The map holds `WedgeFact { wedged_since_unix_s, wedge_class }` per machine, keyed by machine name. Precedence rule: if both triggers fire on the same machine, class `"node"` wins (guest-side actions cannot reach a silent node-agent).
3. **Status surface (`port-runtime` + `port-cli`)** — the machine status contract grows `wedged_since: Option<u64>` and `wedge_class: Option<String>`; the aggregator reads the wedge-state map when building each machine's status. The CLI render adds two lines with the usual `(none)` fallback.

## Components

| Component | Purpose | Interface |
|-----------|---------|-----------|
| `ClusterDetectionConfig` | Parsed TOML block with thresholds and defaults. | Read by the detector task at startup; revalidated on config reload. |
| `WedgeFact` | In-memory per-machine record of `(wedged_since_unix_s, wedge_class)`. | Mutated only by the detector task; read by the status aggregator. |
| Detector task | Background loop evaluating triggers on a fixed interval. | Spawned once per control plane; owns the `wedge_state` write lock. |
| Machine status fields | `wedged_since` + `wedge_class` on `HostedFleetNodeStatus` (or the per-machine status struct). | Serialized to JSON; skipped when `None`. |

## Interfaces

<!-- API contracts, message formats, protocols (if this voyage exposes/consumes APIs) -->

## Data Flow

<!-- How data moves through the system; sequence diagrams if helpful -->

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|

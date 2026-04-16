# Tier-2 Overlay Recreate And Tier-3 Host Recycle - Software Design Description

> Deliver tier-2 overlay recreate action with graceful skip for non-overlay machines, and tier-3 host recycle gated behind the single-tenant host check and a per-provider host_reboot integration (AWS EC2 reboot, SSH systemctl restart). Default off.

**SRS:** [SRS.md](SRS.md)

## Overview

Second and third rungs of the ladder, plus the per-provider reboot plumbing they share. Tier-2 is a surgical "force a clean filesystem" step: drop the rootfs overlay and relaunch; graceful skip for machines not using an overlay. Tier-3 is the blunt last-resort: reboot the entire host via a per-provider integration, gated on a single-tenant check because rebooting a multi-tenant host would take out innocent bystanders.

The `HostRebootClient` trait lands in this voyage because tier-3 needs it; voyage VGzxoN8WF's integration proof reuses the same trait. The trait's shape stays narrow (`reboot(host) -> RebootOutcome`) so adding a provider later is additive.

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

Three logical pieces, two touching the recovery runner and one new module:

1. **Promotion logic** — the runner's existing per-machine state machine gains two more transitions: `tier_1 → tier_2` (on `recovery_attempts.tier_1 >= tier_2_after_attempts` within `window_seconds`) and `tier_2 → tier_3` (on cumulative attempts reaching `tier_3_after_attempts`). Each transition re-reads the wedge state and checks the tier-specific gate before acting.
2. **Tier-2 executor** — a thin helper on the node-agent side that removes `runtime/<machine>/overlay` (idempotent: missing overlay is not an error) and invokes the existing launch path. The filesystem operation is isolated so it can be unit-tested against a fake runtime root.
3. **`HostRebootClient` (`port-runtime::hosted_control_plane::host_reboot`)** — new module exposing a `HostRebootClient` trait and two implementations:
   - `AwsEc2RebootClient` — wraps `aws-sdk-ec2::Client::reboot_instances`, identifies the host's instance via `host.provider_instance_id`, and returns a structured outcome.
   - `SshSystemdRestartClient` — opens an SSH session (reusing the existing SSH host credential plumbing) and runs `systemctl restart port-node-agent`.

Tier-3 holds a host-level lock for the duration of reboot + re-registration wait. The lock also blocks tier-1/2 runners for the host's other machines — important because `port machine launch` during a host reboot would race with the node-agent restarting.

## Components

| Component | Purpose | Interface |
|-----------|---------|-----------|
| Recovery runner promotion | State machine transitions `tier_1 → 2`, `tier_2 → 3`. | Reads `recovery_attempts` and `window_seconds`; writes `last_recovery_action`, `recovery_state`. |
| Tier-2 overlay executor | Drops `runtime/<machine>/overlay` then relaunches. Idempotent. | Node-agent-local; no new external API. |
| `HostRebootClient` trait | Uniform `reboot(host)` interface. | Two implementations: `AwsEc2RebootClient`, `SshSystemdRestartClient`. |
| Host-level reboot lock | Prevents tier-1/2 from racing a tier-3 reboot. | Held by the runner for the duration of reboot + re-registration wait. |
| Doctor checks | `port doctor` validates provider-specific reboot prerequisites. | AWS: credentials present + `ec2:RebootInstances` reachable. SSH: the usual host credential check. |

## Interfaces

<!-- API contracts, message formats, protocols (if this voyage exposes/consumes APIs) -->

## Data Flow

<!-- How data moves through the system; sequence diagrams if helpful -->

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|

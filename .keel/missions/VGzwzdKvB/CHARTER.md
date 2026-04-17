# Hosted Fleet Auto-Recovery For Wedged MicroVM Guests - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver epic `VGzxKV9OX` so Port introduces a guest-agent heartbeat signal and a configurable wedge detector, surfacing `wedged_since`, `wedge_class`, `recovery_attempts`, `last_recovery_action`, and `recovery_state` on `port cluster status --format json` without taking any recovery action yet. | board: VGzxKV9OX |
| MG-02 | Deliver epic `VGzxMc4G4` so Port owns a per-cluster opt-in recovery ladder: tier-1 guest restart and tier-2 overlay recreate fire inside Port against the runtime root; tier-3 surfaces as a structured `awaiting_tier_3_host_recycle` signal for an external consumer (spoke-sh/infra, an operator, or a systemd watcher) to act on. Port also owns serialization against in-flight human lifecycle operations, a sticky `recovery_exhausted` terminal state, and a `port machine unfence` reset path. | board: VGzxMc4G4 |

## Constraints

- Recovery features default to `enabled = false` per cluster; production opts in explicitly.
- Port does not call cloud provider APIs. Tier-3 host recycle is signalled (structured event + `recovery_state = "awaiting_tier_3_host_recycle"`) for an external consumer to act on; AWS/GCP/Azure/SSH integrations live in spoke-sh/infra or another consumer repo, never inside Port.
- `recovery_exhausted` is sticky across windows and clears only via explicit `port machine unfence` or a successful operator-driven launch that produces a Live guest-agent heartbeat.
- Port owns all recovery *actions* that touch the runtime root and detached-forward manifests (tier-1 stop/launch, tier-2 overlay drop); tier-3 is a signal, not an action.
- No cross-cell rebalancing and no guest-side kernel watchdog (the latter belongs in the Spoke guest image, not Port).
- Guest-agent heartbeat (epic VGzxKV9OX) must land and stabilize before recovery actions (epic VGzxMc4G4) are wired up.

## Halting Rules

- DO NOT halt while `recovery_state` is unimplemented or while any tier action is wired up before the guest-agent heartbeat prerequisite is live.
- HALT when a simulated guest-side wedge on a local Firecracker machine converges under tier-1 without operator intervention AND a simulated node-side wedge escalates to `recovery_state = "awaiting_tier_3_host_recycle"` with a structured event emitted for consumer handoff, and returns to `ok` once node-agent re-registration and a fresh guest heartbeat are observed.
- YIELD if the remaining blocker is a product decision about the tier-3 signal shape or consumer handoff protocol (e.g. whether the signal should also carry host-provider hints so downstream consumers can fan out per provider).

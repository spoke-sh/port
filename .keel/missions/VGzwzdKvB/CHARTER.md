# Hosted Fleet Auto-Recovery For Wedged MicroVM Guests - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver epic `VGzxKV9OX` so Port introduces a guest-agent heartbeat signal and a configurable wedge detector, surfacing `wedged_since`, `wedge_class`, `recovery_attempts`, `last_recovery_action`, and `recovery_state` on `port cluster status --format json` without taking any recovery action yet. | board: VGzxKV9OX |
| MG-02 | Deliver epic `VGzxMc4G4` so Port owns a per-cluster opt-in recovery ladder (tier-1 guest restart, tier-2 overlay recreate, tier-3 host recycle behind a single-tenant gate), a per-provider `host_reboot` integration reused by tier-3, serialization against in-flight human lifecycle operations, a sticky `recovery_exhausted` terminal state, and a `port machine unfence` reset path. | board: VGzxMc4G4 |

## Constraints

- Recovery features default to `enabled = false` per cluster; production opts in explicitly.
- Tier-3 host recycle must require a single-tenant host gate; otherwise the wedged machine goes straight to `recovery_exhausted`.
- `recovery_exhausted` is sticky across windows and clears only via explicit `port machine unfence` or a successful operator-driven launch that produces a Live guest-agent heartbeat.
- Port owns all recovery actions inside the process that owns the runtime root and detached-forward manifests; out-of-band callers are not allowed to shell lifecycle commands.
- No cross-cell rebalancing and no guest-side kernel watchdog (the latter belongs in the Spoke guest image, not Port).
- Guest-agent heartbeat (epic VGzxKV9OX) must land and stabilize before recovery actions (epic VGzxMc4G4) are wired up.

## Halting Rules

- DO NOT halt while `recovery_state` is unimplemented or while any tier action is wired up before the guest-agent heartbeat prerequisite is live.
- HALT when a simulated guest-side wedge on a local Firecracker machine converges under tier-1 without operator intervention AND a simulated node-side wedge on a single-tenant host converges under tier-3, with `wedged_since`, `wedge_class`, `recovery_attempts`, `last_recovery_action`, and `recovery_state` visible in `port cluster status --format json`.
- YIELD if the remaining blocker is a product decision about host-provider reboot integration scope (e.g. whether SSH-provider hosts require a manual unfence path instead of an automated recycle).

# Raise Hosted Guest Recovery Fidelity - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver epic `VHUlA6Lhd` so hosted K3s agents and servers self-heal when their healthchecks fail, guest wedge classification stops inferring heartbeat loss from machine placement age, and the machine wedge endpoint exposes enough runtime evidence to safely drive automated recovery decisions. | board: VHUlA6Lhd |

## Constraints

- Hosted recovery behavior must stay opt-in through existing cluster recovery configuration; this slice hardens the enabled path rather than changing default rollout posture.
- Guest wedge evidence must prefer real heartbeat/runtime facts over inferred host placement age; missing guest heartbeat metadata is not itself proof of a guest wedge.
- The `/v1/machines/<name>/wedge` endpoint must report the concrete runtime evidence Port is using so downstream auto-recovery can distinguish recoverable guest failures from stale or missing telemetry.
- The fix must preserve existing recovery ownership boundaries: Port owns guest/runtime supervision and wedge classification, while downstream repos consume the structured signal.

## Halting Rules

- DO NOT halt while hosted K3s services still require operator intervention after an unhealthy healthcheck or while healthy long-lived machines can still classify as `guest` wedged without live runtime evidence.
- HALT when the hosted recovery policy, wedge classifier, and wedge endpoint all reflect the hardened contract and the validating test surface passes.
- YIELD only if the remaining blocker is a product-level decision about new wedge evidence fields or recovery semantics that cannot be derived from the current production failure.

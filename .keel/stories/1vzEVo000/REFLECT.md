---
created_at: 2026-03-08T09:08:00
---

# Reflection - Route Hosted CLI And SDK Through Live Transport

## Knowledge

- [1w03m0000](../../knowledge/1w03m0000.md) Preserve Hosted Control Metadata After Node-Local Execution
- [1w03m1000](../../knowledge/1w03m1000.md) Prove Transport Cutovers With Divergent Client And Server Configs

## Observations

The cutover itself was straightforward once the live hosted client existed in `port-sdk`, but the verification gap was real: several tests were still proving the old config-backed shortcut instead of the transport the story claimed to ship.

The most important defect I found during the wider verification pass was in the node-agent response layer. The HTTP transport worked, but the node agent reused localized runtime helpers and returned local control metadata until a projection layer re-applied the hosted control contract and route context. That would have been easy to miss without the broadened runtime and CLI verification.

The remaining limits are explicit now instead of being hidden by stale docs. Hosted `machine` plus hosted `guest exec|copy|pty|logs` use the live HTTP path, while hosted `copy` still assumes node-visible host paths in the single-node demo and hosted `forward` still depends on the repo-local listener lifecycle.

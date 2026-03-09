---
created_at: 2026-03-08T19:46:33
---

# Reflection - Route Hosted Detached Forward Lifecycle

## Knowledge

- [1vzQLp000](../../knowledge/1vzQLp000.md) Bogus Client Runtime Roots Are A Strong No-Fallback Hosted CLI Proof

## Observations

This slice stayed small because the prior story already shipped the control
plane and node-agent runtime behavior. The CLI only needed to stop rejecting
hosted detached lifecycle flags, route those verbs to the new runtime helpers,
and print the same operator-facing fields the local lane already exposed.

The strongest regression proof was the split hosted config: the server uses the
real node runtime root, while the CLI client config points that runtime root at
a bogus path. Successful hosted `--list` and `--stop` commands under that setup
prove the CLI is no longer reading local runtime state on the client side.

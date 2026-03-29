# Ship Simple Port Cluster Bootstrap Surface - Decision Log

<!-- Append entries below. Each entry is an H2 with ISO timestamp. -->
<!-- Use `keel mission digest` to compress older entries when this file grows large. -->

## 2026-03-28T19:40:00Z

- Created this mission after the downstream `infra` repo proved that Port's
  current hosted-K3s contract is still too low-level for a clean operator
  experience.
- The concrete gap is that infra had to manage control-plane daemons, node
  agents, machine launches, guest install commands, join-token capture, API
  forwarding, and kubeconfig rewriting itself.
- This mission reframes the next Port slice around a simple cluster-first
  operator contract inspired by Slicer's K3s workflow.

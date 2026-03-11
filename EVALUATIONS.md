# Evaluation Guide

Port uses evaluation in the broad product sense: command proofs, board
evidence, and lane-specific validation that demonstrate the product still
behaves as claimed.

## Default Verification Path

For repository-level confidence, run:

```bash
just mission
```

That covers:

- formatting
- workspace tests
- doctests
- board health
- current mission signal

## Evidence Expectations

Every story should leave behind:

- automated tests where practical
- command proofs for user-facing surfaces
- board-linked evidence logs under the story directory
- documentation changes when the operator contract changes

## Verification Matrix

| Area | Primary signal |
|------|----------------|
| Repo health | `just mission`, `just keel doctor` |
| Local Firecracker | `port doctor`, local artifact build, `machine launch|status|stop` |
| Hosted standard lane | control-plane plus node-agent round trip |
| Cloud Hypervisor | lane-specific local or hosted round trip |
| AVF | AVF launcher plus canonical `machine` and `guest` verbs |
| Firecracker/PVM | prepared-node contract and explicit host-kit validation |
| Services | `port service secret|apply|status|stop` plus runtime-state projection |
| Docs and help | command proofs and targeted doc audits |

## Documentation And Help Checks

Top-level docs are part of the product surface.

- `README.md` should stay short and point to canonical detail.
- `CONFIGURATION.md` should hold detailed config and workflow examples.
- `port --help` should remain concise and useful at a glance.

## When To Add New Proofs

Add or update proofs when a change affects:

- the operator-visible command contract
- config shape or environment-variable behavior
- runtime ownership or hosted routing
- platform support boundaries
- artifact or service lifecycle semantics

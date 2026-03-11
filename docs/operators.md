# Operator Guide

Use `port` for runtime workflows and `just mission` for a repo-level mission
report with recent achievements and human-facing artifacts.

## Platform Summary

| Environment | Supported path |
|-------------|----------------|
| Linux | Local Firecracker plus hosted control-plane demos |
| macOS | AVF local workflow through the same `machine` and `guest` verbs |
| Windows | Linux-backed workflow through WSL or a remote Linux host |

## Common Examples

```bash
port doctor
port --config examples/port.toml machine launch --machine demo
PORT_DEMO_TOKEN=demo-token port --config examples/port.toml machine status --machine cloud-aws
```

## Where To Go Next

- Detailed config edits and longer examples:
  [`CONFIGURATION.md`](../CONFIGURATION.md)
- Hosted control-plane and service workflows:
  [`hosted.md`](hosted.md)
- Cloud lanes and provider boundaries:
  [`cloud.md`](cloud.md)
- Artifact references and backend contracts:
  [`artifacts.md`](artifacts.md)
- Firecracker/PVM:
  [`pvm.md`](pvm.md)
- Apple Virtualization Framework:
  [`avf.md`](avf.md)

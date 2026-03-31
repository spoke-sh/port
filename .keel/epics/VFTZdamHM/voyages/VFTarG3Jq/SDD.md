# Default-On Networking Activation Fix - Software Design Description

> Make networking default-on by resolving None to MachineNetworkSpec::default() in the Firecracker launch path

**SRS:** [SRS.md](SRS.md)

## Overview

Two-file fix: add `Default` impl for `MachineNetworkSpec` in port-model, then resolve `machine.network` from `Option` to effective config using `unwrap_or_default()` in three places in the Firecracker launch path.

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Default strategy | `unwrap_or_default()` in runtime, not changing the model to non-optional | Keeps model backward-compatible; only local Firecracker path uses defaults |
| DNS defaults | 8.8.8.8, 8.8.4.4 in both `Default` impl and serde default | Every code path that produces a `MachineNetworkSpec` gets working DNS |

## Components

### MachineNetworkSpec Default (port-model)

Add `impl Default` and `default_dns_servers()` serde function so both the programmatic default and the serde-deserialized default include DNS servers.

### Effective Network Resolution (port-runtime)

In `firecracker_local_launch_machine()`, compute `effective_network = machine.network.clone().unwrap_or_default()` and use it for:
1. `setup_host_networking()` call
2. `build_firecracker_config()` call
3. Network state persistence to `network-state.json`

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| TAP setup fails (permissions) | `setup_host_networking` returns error | VM launch fails with clear message | Operator runs Port with sufficient privileges |

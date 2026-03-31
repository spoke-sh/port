# Complete Guest VM Outbound Networking - Decision Log

<!-- Append entries below. Each entry is an H2 with ISO timestamp. -->
<!-- Use `keel mission digest` to compress older entries when this file grows large. -->

## 2026-03-31T12:56:28

Fixed default-on networking activation. Root cause: MachineSpec.network was Option with no Default — when TOML lacked [machines.demo.network], all networking code was skipped. Fix: added Default impl for MachineNetworkSpec (enabled=true, 172.16.0.0/24, DNS 8.8.8.8/8.8.4.4), added default_dns_servers() serde function, and changed firecracker_local_launch_machine() to resolve machine.network via unwrap_or_default(). Test sample_machine() explicitly disables networking to avoid TAP creation in non-root test environments. All 54 model tests and 130/132 runtime tests pass (2 pre-existing failures).

## 2026-03-31T12:56:33

Mission achieved by local system user 'alex'

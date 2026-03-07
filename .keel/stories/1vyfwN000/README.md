---
id: 1vyfwN000
title: Connect Exec Pty And Logs To Live VMs
type: feat
status: done
created_at: 2026-03-06T16:54:47
updated_at: 2026-03-06T17:11:36
scope: 1vydg7000/1vyfve000
started_at: 2026-03-06T17:02:45
submitted_at: 2026-03-06T17:11:33
completed_at: 2026-03-06T17:11:36
---

# Connect Exec Pty And Logs To Live VMs

## Summary

Expose the guest agent on a real guest control port and make `port guest exec`,
`pty`, and `logs` use that live transport automatically for launched
Firecracker VMs.

## Acceptance Criteria

<!-- verify: manual, SRS-02:start:end, proof: ac-1.log-->
- [x] [SRS-02/AC-01] The built guest image launches `port-guest-agent` on the configured guest control port in addition to the Unix-socket test path. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "port.guest_control_port|--vsock-port" scripts/artifacts/build-guest-image.sh && nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime firecracker_config_contains_kernel_rootfs_and_vsock', proof: ac-1.log-->
<!-- verify: manual, SRS-03:start:end, proof: ac-2.log-->
- [x] [SRS-03/AC-01] `port guest exec --machine demo -- ...` succeeds against a launched VM through the canonical CLI and model. <!-- [SRS-03/AC-01] verify: nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime guest_exec_uses_firecracker_vsock_tunnel_when_runtime_socket_is_absent, proof: ac-2.log-->
<!-- verify: manual, SRS-03:start:end, proof: ac-3.log-->
- [x] [SRS-03/AC-02] `port guest pty --machine demo -- ...` and `port guest logs --machine demo --path ...` both succeed against a launched VM through the canonical CLI and model. <!-- [SRS-03/AC-02] verify: nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime -p port-guest-agent -p port-cli, proof: ac-3.log-->
<!-- verify: manual, SRS-03:start:end, proof: ac-4.log-->
- [x] [SRS-03/AC-03] Automated tests cover transport selection plus the Firecracker-vsock control handshake without requiring a real VM. <!-- [SRS-03/AC-03] verify: nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime guest_exec_uses_firecracker_vsock_tunnel_when_runtime_socket_is_absent, proof: ac-4.log-->

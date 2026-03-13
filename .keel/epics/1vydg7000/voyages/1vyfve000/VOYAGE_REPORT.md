# VOYAGE REPORT: Wire Live Guest Transport

## Voyage Metadata
- **ID:** 1vyfve000
- **Epic:** 1vydg7000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Rework Copy And Forward For Live Guest Transport
- **ID:** 1vyfvx000
- **Status:** done

#### Summary
Replace the shared-host-path and guest-local-listener assumptions in `copy` and
`forward` with behaviors that stay coherent across a real host/guest boundary.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] `port guest copy --direction host-to-guest` transfers file contents into the launched VM without requiring the guest to see the host source path. <!-- [SRS-04/AC-01] verify: nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime copy_guest_file_uses_firecracker_vsock_tunnel_in_both_directions, proof: ac-1.log-->
- [x] [SRS-04/AC-02] `port guest copy --direction guest-to-host` transfers file contents back to the host through the canonical CLI and model. <!-- [SRS-04/AC-02] verify: nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-cli cli_guest_commands_cover_all_capabilities, proof: ac-2.log-->
- [x] [SRS-05/AC-01] `port guest forward` binds and serves on the host side, proxies to a guest target through the live transport, and behaves according to documented lifecycle expectations. <!-- [SRS-05/AC-01] verify: nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime guest_forward_session_proxies_through_firecracker_vsock_tunnel, proof: ac-3.log-->
- [x] [SRS-06/AC-01] `port --help`, README, and operator docs describe the live guest transport and the current `guest forward` lifecycle accurately. <!-- [SRS-06/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-cli help_includes_primary_surfaces && rg -n "foreground host-side proxy|ifconfig lo up" README.md docs/operators.md crates/port-cli/src/lib.rs', proof: ac-4.log-->

#### Implementation Insights
- **1vygVp000: Preserve buffered bytes when switching from framed control to raw streams**
  - Insight: A `BufReader` can prefetch guest data that arrives immediately after the final framed response. Dropping the reader with `into_inner()` without draining `buffer()` silently loses those bytes.
  - Suggested Action: When handing a Port transport from framed JSON into raw copy/forward mode, either avoid buffered reads at the handoff point or wrap the underlying stream with a prefix reader that drains the buffered bytes first.
  - Applies To: `crates/port-runtime/src/lib.rs`, future guest transport/proxy code
  - Category: architecture


#### Verified Evidence
- [ac-1.log](../../../../stories/1vyfvx000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vyfvx000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vyfvx000/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/1vyfvx000/EVIDENCE/ac-4.log)

### Stabilize Runtime State For Guest Transport
- **ID:** 1vyfwJ000
- **Status:** done

#### Summary
Harden the local runtime surface that the live guest transport depends on:
clean stale runtime state before relaunch and replace the generic missing-socket
guest error with an actionable launched-VM transport diagnostic.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `port machine launch` removes stale runtime pid/vsock/socket files left by dead Firecracker runs before attempting a relaunch. <!-- [SRS-01/AC-01] verify: nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime prepare_runtime_state_cleans_stale_socket_and_pid_files, proof: ac-1.log-->
- [x] [SRS-01/AC-02] `port machine launch` fails with an explicit "already running" message when the requested machine still has a live Firecracker process under the same runtime root. <!-- [SRS-01/AC-02] verify: nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime prepare_runtime_state_rejects_live_matching_firecracker_process, proof: ac-2.log-->
- [x] [SRS-01/AC-03] `port guest ...` against a launched VM without a connected live transport returns an actionable error that points at the launched-VM transport gap instead of only reporting a missing host socket. <!-- [SRS-01/AC-03] verify: nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime guest_operations_explain_missing_live_vm_transport, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vyfwJ000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vyfwJ000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vyfwJ000/EVIDENCE/ac-3.log)

### Connect Exec Pty And Logs To Live VMs
- **ID:** 1vyfwN000
- **Status:** done

#### Summary
Expose the guest agent on a real guest control port and make `port guest exec`,
`pty`, and `logs` use that live transport automatically for launched
Firecracker VMs.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] The built guest image launches `port-guest-agent` on the configured guest control port in addition to the Unix-socket test path. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && rg -n "port.guest_control_port|--vsock-port" scripts/artifacts/build-guest-image.sh && nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime firecracker_config_contains_kernel_rootfs_and_vsock', proof: ac-1.log-->
- [x] [SRS-03/AC-01] `port guest exec --machine demo -- ...` succeeds against a launched VM through the canonical CLI and model. <!-- [SRS-03/AC-01] verify: nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime guest_exec_uses_firecracker_vsock_tunnel_when_runtime_socket_is_absent, proof: ac-2.log-->
- [x] [SRS-03/AC-02] `port guest pty --machine demo -- ...` and `port guest logs --machine demo --path ...` both succeed against a launched VM through the canonical CLI and model. <!-- [SRS-03/AC-02] verify: nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime -p port-guest-agent -p port-cli, proof: ac-3.log-->
- [x] [SRS-03/AC-03] Automated tests cover transport selection plus the Firecracker-vsock control handshake without requiring a real VM. <!-- [SRS-03/AC-03] verify: nix develop -c env CARGO_TARGET_DIR=/tmp/port-target cargo test -p port-runtime guest_exec_uses_firecracker_vsock_tunnel_when_runtime_socket_is_absent, proof: ac-4.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vyfwN000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vyfwN000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vyfwN000/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/1vyfwN000/EVIDENCE/ac-4.log)



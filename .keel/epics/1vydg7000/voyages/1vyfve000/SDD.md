# Wire Live Guest Transport - Software Design Description

> Make the canonical port guest flows work against launched Firecracker VMs through a live guest transport

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage introduces a real host-to-guest control path for launched
Firecracker VMs and aligns the CLI behavior with the actual host/guest
boundary:

- `port-guest-agent` serves both the existing Unix-socket test path and a
  guest-side vsock control port inside the VM.
- `port-runtime` selects a transport per machine: host runtime socket when
  present, otherwise the Firecracker vsock tunnel at
  `<runtime-root>/<machine>/guest.vsock`.
- `exec`, `pty`, and `logs` continue to use simple request/response framing.
- `copy` changes from shared-path semantics to byte transfer across the
  transport.
- `forward` becomes a host-side foreground proxy that uses the guest transport
  per inbound connection rather than asking the guest agent to bind a listener
  that the host cannot reach.

## Context & Boundaries

### In Scope

- guest-agent boot/init wiring inside the built guest image;
- runtime transport selection and Firecracker vsock handshakes;
- protocol changes required for streamed copy and proxy forwarding;
- CLI/help/doc updates for the new live-VM behavior.

### Out of Scope

- Firecracker REST API support;
- remote/cloud launch execution;
- a background forwarding daemon or machine-listing surface.

```
┌────────────────────────────────────────────────────────────────┐
│                           port CLI                             │
│    guest exec/copy/pty/logs/forward from canonical surface     │
└───────────────────────────────┬────────────────────────────────┘
                                │
                   ┌────────────▼────────────┐
                   │       port-runtime      │
                   │ transport selection +   │
                   │ copy/forward host logic │
                   └────────────┬────────────┘
                                │
                 ┌──────────────┴──────────────┐
                 │                             │
      ┌──────────▼──────────┐      ┌──────────▼──────────┐
      │ runtime guest socket│      │ Firecracker vsock   │
      │  (tests/local shim) │      │ tunnel guest.vsock  │
      └──────────┬──────────┘      └──────────┬──────────┘
                 │                             │
                 └──────────────┬──────────────┘
                                │
                    ┌───────────▼───────────┐
                    │    port-guest-agent   │
                    │ unix + vsock listeners│
                    └───────────────────────┘
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Firecracker host vsock UDS | runtime interface | tunnel host requests into guest vsock ports without the Firecracker API | current Firecracker UDS `CONNECT <port>` behavior |
| `port-agent-protocol` | workspace crate | shared request/response and byte-stream framing | workspace current |
| `vsock` Rust crate or equivalent Linux AF_VSOCK binding | guest agent runtime | accept guest control connections inside the VM | current crate API |
| built guest image `/init` | artifact pipeline | launch `port-guest-agent` with the configured listeners | current build script |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Live control channel | Use Firecracker’s host-side vsock tunnel plus the machine `control_port` | Works with `--no-api` and matches the current model |
| Transport selection | Prefer the explicit runtime Unix socket when present; otherwise connect to the launched VM transport | Preserves testability while making launched VMs first-class |
| Copy semantics | Transfer file bytes over the guest transport instead of relying on shared filesystem visibility | Real VMs do not share host paths with the guest agent |
| Forward semantics | Run forwarding on the host in the foreground CLI process and use the guest agent only for guest-side target connections | A guest-local listener is not a usable host operator surface |
| Forward lifecycle | Foreground/attached for the MVP | Avoids hidden daemons while keeping the behavior honest and testable |

## Architecture

The runtime gains a transport abstraction that can open a control connection to
either:

1. the existing runtime guest-agent Unix socket, or
2. the Firecracker guest-vsock tunnel for the machine’s configured
   `control_port`.

The guest agent grows a second serving path for vsock alongside the existing
Unix socket path. The protocol keeps newline-delimited JSON frames for control
messages and uses the existing `Accepted` response shape to switch specific
operations into raw byte streaming when needed.

## Components

- `port-guest-agent`
  serves requests over Unix sockets for tests and over vsock inside the guest;
  handles request/response operations plus copy/forward streaming behavior.
- `port-runtime`
  resolves machine transport, performs the Firecracker UDS handshake, and owns
  host-side behaviors such as local file reads/writes and foreground port
  forwarding.
- `port-agent-protocol`
  remains the single source of truth for operation envelopes and gains the
  stream metadata needed for transfer-oriented operations.
- `build-guest-image.sh`
  launches the guest agent with the configured vsock listener so the built
  image is reachable after boot.
- CLI/docs
  surface the actual launched-VM guest transport and the new forward lifecycle.

## Interfaces

- Firecracker transport handshake:
  host connects to `<runtime-root>/<machine>/guest.vsock`, writes
  `CONNECT <control-port>\n`, waits for an `OK` line, then speaks the guest
  protocol over that socket.
- Guest control protocol:
  newline-delimited JSON frames for request/response; specific operations may
  transition into a raw byte stream after an `Accepted` response.
- CLI:
  `port guest exec/copy/pty/logs/forward` remain the canonical operator entry
  points. `forward` is documented as a foreground proxy workflow.

## Data Flow

1. `port` loads the machine config and resolves `guest.control_port`.
2. `port-runtime` selects a transport:
   - runtime socket if explicitly present, or
   - Firecracker vsock tunnel if the VM is launched.
3. For `exec`, `pty`, and `logs`, runtime sends one request frame and reads one
   final response frame.
4. For `copy`, runtime and guest agent coordinate a byte-transfer phase so the
   host writes or reads the local file and the guest agent writes or reads the
   guest file.
5. For `forward`, the CLI binds a host listener, and each inbound client causes
   a fresh guest-transport request that bridges that host socket to the guest
   target.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Relaunch collides with stale runtime files | pid/socket inspection before spawn | remove stale files or fail if the original Firecracker process is still live | rerun launch or choose another runtime root |
| Guest transport unavailable after launch | missing runtime socket plus failed Firecracker-vsock handshake | return an error that points at the launched VM transport and its runtime artifacts | inspect runtime logs or rebuild the guest image |
| Copy direction still assumes shared filesystem visibility | automated tests on host/guest transfer paths | replace path-sharing logic with transport byte transfer | use runtime-owned file I/O on the host side |
| Forward binds in the wrong place | CLI and launched-VM proof | keep the listener on the host and make the guest side connect only to the guest target | rerun `port guest forward` from the host |

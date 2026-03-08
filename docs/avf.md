# Apple Virtualization Framework Contract

Port treats Apple Virtualization Framework as the first-class macOS substrate
lane. The goal is not a second operator model. The goal is to map the existing
Port lifecycle and guest verbs onto AVF's native primitives.

## Runtime Contract

The AVF lane should keep the canonical Port verbs and reinterpret only the
substrate-specific implementation beneath them.

| Port concern | AVF primitive | Port contract |
|--------------|---------------|---------------|
| `machine launch` | `VZVirtualMachineConfiguration` plus Linux boot loader | Local Port runtime owns launch today; future hosted macOS nodes can reuse the same lifecycle contract through a node agent |
| guest agent transport | virtio socket devices and listeners | Keep the current guest protocol and map it onto AVF virtio sockets instead of inventing a macOS-specific guest API |
| console and boot logs | serial-port configuration | Capture guest console output through AVF serial ports so `machine status` and logs stay coherent |
| optional host/guest file exchange | directory sharing devices | Useful for operator workflows and Rosetta support, but not a replacement for Port's guest `copy` semantics |

## Launch Ownership

Port's AVF launch-ownership contract is:

- local macOS workflow: the `port` runtime owns the AVF VM directly
- hosted macOS workflow: a future node agent owns the AVF VM on behalf of the
  control plane
- the operator-facing verbs remain `machine launch`, `list`, `status`, `stop`,
  and guest `exec`, `copy`, `pty`, `logs`, `forward`

Current runtime foundation:

- `port machine launch|status|stop` now route AVF-targeted machines through a
  local AVF driver instead of failing at driver selection time
- the current driver expects an explicit launcher helper, but that helper now
  receives a canonical runtime contract including the runtime root, guest-agent
  socket path, and console-log path
- AVF-backed `guest exec|copy|pty|logs|forward` now attach through the canonical
  runtime `guest-agent.sock` once the launcher helper exposes the transport
  bridge
- AVF boot output now lands in the canonical runtime console log so `machine`
  inspection surfaces can reference it

That is the same ownership rule Port now uses for Linux and hosted design work:
one operator model, different runtime owners.

## Guest Transport Mapping

Port keeps the guest agent and its protocol surface on AVF.

Required mapping:

- guest `exec`, `copy`, `pty`, `logs`, and `forward` travel over AVF virtio
  sockets
- the launcher/helper is responsible for bridging that transport onto Port's
  canonical runtime `guest-agent.sock`
- console capture uses AVF serial ports and lands in the runtime console-log
  path, not the guest-agent socket

That means the transport changes, but the guest protocol does not.

## Operator Workflow

The AVF macOS lane should be discoverable to operators as a native macOS path.

Required operator expectations:

- run the AVF lane on macOS
- use the same canonical `port` commands instead of a macOS-only command tree
- treat AVF as `standard` protection only; Port does not define an AVF/PVM lane
- expect optional directory sharing for operator convenience and Rosetta
  workflows, not as the primary guest-control surface

macOS distribution boundary:

- local development binaries can use AVF directly
- distributed macOS app targets need Apple's virtualization entitlement
- sandboxed distributions also need the relevant network and file-access
  entitlements for the chosen operator workflow

Rosetta boundary:

- Rosetta support in Linux guests is an Apple-silicon-specific workflow
- it depends on AVF directory sharing plus the guest-side Rosetta service
- it is useful for operator tooling, but not required for the canonical Port
  guest-agent path

## Verification Expectations

Port now ships AVF-focused `port doctor` checks for the first contract slice:

1. Host platform is macOS and the AVF lane is modeled as a local host path.
2. The current host architecture is in the AVF support set (`x86_64` or
   `aarch64`).
3. The build/distribution path is bounded by Apple's virtualization entitlement
   requirements when applicable.

The executable runtime path now has proof for the next contract slice:

4. The AVF-backed guest can be controlled through the shared guest protocol
   over AVF virtio sockets.
5. Console/log capture works through AVF serial ports.

## Follow-On Work

The ordered implementation sequence after this contract is:

1. Publish the native macOS AVF workflow across CLI help and operator docs with
   reproducible proof commands.
2. Decide how much of directory sharing and Rosetta support belongs in the
   first executable macOS lane versus later operator-ergonomics slices.

## Research Basis

- Apple Virtualization framework docs:
  <https://developer.apple.com/documentation/virtualization/>
- `VZVirtioSocketDeviceConfiguration`:
  <https://developer.apple.com/documentation/virtualization/vzvirtiosocketdeviceconfiguration>
- `VZVirtioSocketListener`:
  <https://developer.apple.com/documentation/virtualization/vzvirtiosocketlistener>
- `VZVirtioConsoleDeviceSerialPortConfiguration`:
  <https://developer.apple.com/documentation/virtualization/vzvirtioconsoledeviceserialportconfiguration>
- Apple entitlement guidance:
  <https://developer.apple.com/documentation/bundleresources/entitlements/com_apple_security_virtualization>
- Apple Rosetta-in-Linux-VM guidance:
  <https://developer.apple.com/documentation/virtualization/running_intel_binaries_in_linux_vms_with_rosetta>

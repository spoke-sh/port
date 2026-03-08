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

That is the same ownership rule Port now uses for Linux and hosted design work:
one operator model, different runtime owners.

## Guest Transport Mapping

Port keeps the guest agent and its protocol surface on AVF.

Required mapping:

- guest `exec`, `copy`, `pty`, `logs`, and `forward` travel over AVF virtio
  sockets
- the host side listens and attaches through AVF socket listeners instead of
  Firecracker vsock paths
- console capture uses AVF serial ports, not the guest-agent socket

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

Future Port validation for the AVF lane should check all of the following:

1. Host platform is macOS and the AVF APIs are available.
2. The build/distribution path satisfies Apple's virtualization entitlement
   requirements when applicable.
3. The AVF driver can boot the selected Linux guest.
4. The guest agent is reachable through AVF virtio sockets.
5. Console/log capture works through AVF serial ports.

## Follow-On Work

The ordered implementation sequence after this contract is:

1. Add AVF-focused `port doctor` checks for macOS, AVF availability, and
   entitlement/distribution boundaries.
2. Implement an AVF driver that maps machine launch onto AVF VM configuration.
3. Reuse the canonical guest protocol over AVF virtio sockets.
4. Add console/log capture through AVF serial ports.
5. Decide how much of directory sharing and Rosetta support belongs in the
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

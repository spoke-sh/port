# VOYAGE REPORT: Substrate Drivers And Host Kits

## Voyage Metadata
- **ID:** 1vz3j0000
- **Epic:** 1vz3ck000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 4/4 stories complete

## Implementation Narrative
### Extract Firecracker Driver Boundary
- **ID:** 1vz3kq000
- **Status:** done

#### Summary
Define and scaffold the first substrate-driver boundary so local Firecracker
runtime ownership becomes one implementation behind shared lifecycle and guest
attach interfaces.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `port-runtime` defines implementation-ready driver seams for launch, inventory/status, stop, and guest attach without hiding Firecracker-specific behavior behind ad hoc branching. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && env CARGO_TARGET_DIR=/tmp/port-target cargo test -q -p port-runtime && rg -n "trait MachineDriver|struct FirecrackerLocalDriver|fn driver_for_machine|fn firecracker_local_launch_machine|fn firecracker_local_list_machines|fn firecracker_local_stop_machine|fn resolve_firecracker_guest_endpoint" crates/port-runtime/src/lib.rs', proof: ac-1.log-->

#### Implementation Insights
- **1vz3uv000: Guest Forward Needs Endpoint-Level Driver Seams**
  - Insight: A driver seam that only exposes `connect()` is not enough for long-lived flows like forwarding; the abstraction has to preserve a reusable guest-endpoint concept so each inbound connection can attach independently.
  - Suggested Action: When extracting additional substrate drivers, model guest attachment as endpoint resolution plus connection, not as a one-shot stream factory.
  - Applies To: `crates/port-runtime/src/lib.rs`, future hosted and AVF guest transport work
  - Category: architecture


#### Verified Evidence
- [ac-1.log](../../../../stories/1vz3kq000/EVIDENCE/ac-1.log)

### Define Hosted Machine Inventory Contract
- **ID:** 1vz3kt000
- **Status:** done

#### Summary
Define the first hosted machine inventory and lifecycle contract so the current
`machine list|status|stop` verbs can target local runtime roots or future
node-agent-backed ownership without changing the operator model.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] Port publishes implementation-ready lifecycle and inventory contracts for local versus hosted ownership, including how machine status is sourced and routed. <!-- [SRS-02/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz3kt000/verify-ac-1.sh, proof: ac-1.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vz3kt000/EVIDENCE/ac-1.log)

### Define AVF Execution Contract
- **ID:** 1vz3l2000
- **Status:** done

#### Summary
Define the first Apple Virtualization Framework execution contract for Port,
covering launch ownership, guest transport mapping, and operator workflow on
macOS.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] Port defines the AVF runtime contract, including how canonical lifecycle and guest operations map onto AVF-specific primitives. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz3l2000/verify-ac-1.sh, proof: ac-1.log-->
- [x] [SRS-05/AC-01] The story produces an implementation-ready AVF follow-on slice with explicit docs and verification expectations for macOS operators and leaves the voyage with a coherent ordered implementation set. <!-- [SRS-05/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz3l2000/verify-ac-2.sh, proof: ac-2.log-->

#### Implementation Insights
- **1vz4B5000: AVF Should Keep The Guest Protocol**
  - Insight: AVF does not require a second guest API. Port can keep the existing guest-agent model by mapping guest transport onto virtio sockets, console capture onto serial ports, and treating directory sharing as optional operator ergonomics rather than the control plane.
  - Suggested Action: Implement the AVF driver around virtio sockets plus serial ports first, then add directory sharing and Rosetta as explicit optional workflows.
  - Applies To: `crates/port-model/src/lib.rs`, `docs/avf.md`, future AVF driver and macOS `port doctor` work
  - Category: architecture


#### Verified Evidence
- [ac-1.log](../../../../stories/1vz3l2000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vz3l2000/EVIDENCE/ac-2.log)

### Plan Pvm Host Kit
- **ID:** 1vz3lA000
- **Status:** done

#### Summary
Define the first x86_64 PVM host-kit and artifact-kit contract for Port,
including host kernel, VMM, artifact variants, validation, and explicit
operator prerequisites while keeping arm64 Firecracker/PVM research-only.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] Port publishes an implementation-ready host-kit and artifact-kit contract for the x86_64 PVM lane, including prepared host components and validation expectations. <!-- [SRS-03/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz3lA000/verify-ac-1.sh, proof: ac-1.log-->
- [x] [SRS-03/AC-02] The story records an explicit x86_64 keep / arm64 research-only boundary with operator-visible implications and follow-on implementation work. <!-- [SRS-03/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vz3lA000/verify-ac-2.sh, proof: ac-2.log-->

#### Implementation Insights
- **1vz4A6000: PVM Needs Host-Kit Contracts**
  - Insight: The PVM lane is not safely modeled as `protection_mode = "pvm"` on top of the standard Firecracker runtime. It needs an explicit host kit, artifact kit, and validation contract before runtime work is credible.
  - Suggested Action: When implementing PVM follow-on work, start with host-kit packaging and `port doctor` validation before wiring launch behavior.
  - Applies To: `crates/port-model/src/lib.rs`, `docs/pvm.md`, future `port doctor` and Firecracker/PVM driver work
  - Category: architecture


#### Verified Evidence
- [ac-1.log](../../../../stories/1vz3lA000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vz3lA000/EVIDENCE/ac-2.log)



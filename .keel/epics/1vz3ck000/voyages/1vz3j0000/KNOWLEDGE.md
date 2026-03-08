---
created_at: 2026-03-07T19:11:27
---

# Knowledge - 1vz3j0000

> Automated synthesis of story reflections.

## Story Knowledge

## Story: Plan Pvm Host Kit (1vz3lA000)

### 1vz4A6000: PVM Needs Host-Kit Contracts

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Planning Firecracker/PVM support for cloud-cost-controlled Port execution |
| **Insight** | The PVM lane is not safely modeled as `protection_mode = "pvm"` on top of the standard Firecracker runtime. It needs an explicit host kit, artifact kit, and validation contract before runtime work is credible. |
| **Suggested Action** | When implementing PVM follow-on work, start with host-kit packaging and `port doctor` validation before wiring launch behavior. |
| **Applies To** | `crates/port-model/src/lib.rs`, `docs/pvm.md`, future `port doctor` and Firecracker/PVM driver work |
| **Applied** | yes |



---

## Story: Extract Firecracker Driver Boundary (1vz3kq000)

### 1vz3uv000: Guest Forward Needs Endpoint-Level Driver Seams

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Extracting substrate drivers for guest operations in `port-runtime` |
| **Insight** | A driver seam that only exposes `connect()` is not enough for long-lived flows like forwarding; the abstraction has to preserve a reusable guest-endpoint concept so each inbound connection can attach independently. |
| **Suggested Action** | When extracting additional substrate drivers, model guest attachment as endpoint resolution plus connection, not as a one-shot stream factory. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, future hosted and AVF guest transport work |
| **Applied** | yes |



---

## Story: Define AVF Execution Contract (1vz3l2000)

### 1vz4B5000: AVF Should Keep The Guest Protocol

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Defining the first macOS AVF execution contract for Port |
| **Insight** | AVF does not require a second guest API. Port can keep the existing guest-agent model by mapping guest transport onto virtio sockets, console capture onto serial ports, and treating directory sharing as optional operator ergonomics rather than the control plane. |
| **Suggested Action** | Implement the AVF driver around virtio sockets plus serial ports first, then add directory sharing and Rosetta as explicit optional workflows. |
| **Applies To** | `crates/port-model/src/lib.rs`, `docs/avf.md`, future AVF driver and macOS `port doctor` work |
| **Applied** | yes |



---

## Synthesis

### ksN74aql9: PVM Needs Host-Kit Contracts

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Planning Firecracker/PVM support for cloud-cost-controlled Port execution |
| **Insight** | The PVM lane is not safely modeled as `protection_mode = "pvm"` on top of the standard Firecracker runtime. It needs an explicit host kit, artifact kit, and validation contract before runtime work is credible. |
| **Suggested Action** | When implementing PVM follow-on work, start with host-kit packaging and `port doctor` validation before wiring launch behavior. |
| **Applies To** | `crates/port-model/src/lib.rs`, `docs/pvm.md`, future `port doctor` and Firecracker/PVM driver work |
| **Linked Knowledge IDs** | 1vz4A6000 |
| **Score** | 0.92 |
| **Confidence** | 0.97 |
| **Applied** | yes |

### VgE28uXqM: Guest Forward Needs Endpoint-Level Driver Seams

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Extracting substrate drivers for guest operations in `port-runtime` |
| **Insight** | A driver seam that only exposes `connect()` is not enough for long-lived flows like forwarding; the abstraction has to preserve a reusable guest-endpoint concept so each inbound connection can attach independently. |
| **Suggested Action** | When extracting additional substrate drivers, model guest attachment as endpoint resolution plus connection, not as a one-shot stream factory. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, future hosted and AVF guest transport work |
| **Linked Knowledge IDs** | 1vz3uv000 |
| **Score** | 0.77 |
| **Confidence** | 0.93 |
| **Applied** | yes |

### snDcGTtYy: AVF Should Keep The Guest Protocol

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Defining the first macOS AVF execution contract for Port |
| **Insight** | AVF does not require a second guest API. Port can keep the existing guest-agent model by mapping guest transport onto virtio sockets, console capture onto serial ports, and treating directory sharing as optional operator ergonomics rather than the control plane. |
| **Suggested Action** | Implement the AVF driver around virtio sockets plus serial ports first, then add directory sharing and Rosetta as explicit optional workflows. |
| **Applies To** | `crates/port-model/src/lib.rs`, `docs/avf.md`, future AVF driver and macOS `port doctor` work |
| **Linked Knowledge IDs** | 1vz4B5000 |
| **Score** | 0.90 |
| **Confidence** | 0.95 |
| **Applied** | yes |


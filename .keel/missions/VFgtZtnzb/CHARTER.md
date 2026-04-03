# Seal Guest Backed Session Drivers For The Creator Platform - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Port provides a real provider-backed hosted execution lane for `cloud-aws` on prepared AWS x86_64 PVM hosts as the substrate for creator-platform guest sessions. | manual: inspect verified hosted AWS PVM mission VFgcM1Zpu and epic VFgcPDfEj |
| MG-02 | `port control-plane prepare-pvm-node`, `port machine launch --machine cloud-aws`, `port machine status --machine cloud-aws`, and `port machine stop --machine cloud-aws` succeed through the hosted control-plane and node-agent path for the canonical AWS lane. | manual: inspect verified hosted AWS PVM mission VFgcM1Zpu and epic VFgcPDfEj |
| MG-03 | Port exposes stable guest-session identity and driver metadata for guest-backed shell flows so upstream systems can treat Port sessions as one audited shell driver. | board: VFgtgGEog |
| MG-04 | Guest-backed `pty`, `exec`, and `forward` behavior remains canonical and consumable by higher-level product surfaces without inventing a second shell protocol. | board: VFgtgGWoh |
| MG-05 | Failure surfaces stay explicit: missing host kit, missing artifacts, wrong lane, or unstable guest-session metadata fail honestly without silent fallback. | manual: inspect canonical failure paths plus automated tests |

## Constraints

- Scope is AWS x86_64 hosted PVM first.
- Keep the existing Port verb model.
- Do not move creator-facing policy or domain/auth concerns into Port.
- Reuse the verified hosted AWS PVM runtime contract as the substrate baseline; do not fork or duplicate it as a second runtime lane.
- Keep guest-backed shell behavior on the existing Port guest protocol surface rather than inventing a creator-specific shell protocol.

## Halting Rules

- DO NOT halt while creator-platform guest sessions still depend on ad hoc session identifiers, unstable driver labels, or a creator-specific shell protocol on `cloud-aws`.
- HALT when the verified AWS hosted PVM runtime contract plus the attached epics prove one audited Port shell-driver surface for guest-backed `exec`, `pty`, and `forward`, with explicit failure guidance.
- YIELD if the remaining blocker is a product decision about creator-facing identity policy, audit retention semantics, or auth/domain ownership outside Port.

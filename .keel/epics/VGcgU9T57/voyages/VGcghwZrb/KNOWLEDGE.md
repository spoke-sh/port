---
created_at: 2026-04-12T17:45:16
---

# Knowledge - VGcghwZrb

> Automated synthesis of story reflections.

## Story Knowledge

## Story: Enforce Managed Service Ownership For Hosted K3s (VGcgtDfDT)

### VGct0mwbD: Hosted Proof Harnesses Need Isolated Control-Plane State

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | When a proof script starts a hosted control plane and node agents inside this repository workspace |
| **Insight** | Hosted proof harnesses can collide with existing `.port/hosted/<control-plane>` state and stale binary assumptions unless they use a unique temporary control-plane name and resolve the CLI through the active `CARGO_TARGET_DIR`. |
| **Suggested Action** | Give each hosted proof run a unique control-plane name and derive the CLI binary path from `CARGO_TARGET_DIR` before starting long-lived harness processes. |
| **Applies To** | `scripts/render-hosted-*.sh` |
| **Applied** | yes |



---

## Story: Record Hosted Worker Stability Soak Proof (VGcgtFI9v)

### VGct92Y9v: Hosted Proof Harnesses Must Seed Registrations Before Control-Plane Start

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | When a hosted proof script seeds `registered-nodes.json` directly instead of depending on live registration refresh during the run |
| **Insight** | `control-plane serve` loads registered node state into memory at startup, so proof harnesses that hand-author registration state must write it before starting the control plane or the route resolver will treat every candidate node as missing. |
| **Suggested Action** | Reserve node bind addresses first, persist registered-node state with current freshness timestamps, then start `control-plane serve` and the node-agent processes. |
| **Applies To** | `scripts/render-hosted-*.sh`, hosted control-plane proof harnesses |
| **Applied** | yes |



---

## Synthesis

### F2BVZLaix: Hosted Proof Harnesses Need Isolated Control-Plane State

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | When a proof script starts a hosted control plane and node agents inside this repository workspace |
| **Insight** | Hosted proof harnesses can collide with existing `.port/hosted/<control-plane>` state and stale binary assumptions unless they use a unique temporary control-plane name and resolve the CLI through the active `CARGO_TARGET_DIR`. |
| **Suggested Action** | Give each hosted proof run a unique control-plane name and derive the CLI binary path from `CARGO_TARGET_DIR` before starting long-lived harness processes. |
| **Applies To** | `scripts/render-hosted-*.sh` |
| **Linked Knowledge IDs** | VGct0mwbD |
| **Score** | 0.86 |
| **Confidence** | 0.92 |
| **Applied** | yes |

### m9prKplYZ: Hosted Proof Harnesses Must Seed Registrations Before Control-Plane Start

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | When a hosted proof script seeds `registered-nodes.json` directly instead of depending on live registration refresh during the run |
| **Insight** | `control-plane serve` loads registered node state into memory at startup, so proof harnesses that hand-author registration state must write it before starting the control plane or the route resolver will treat every candidate node as missing. |
| **Suggested Action** | Reserve node bind addresses first, persist registered-node state with current freshness timestamps, then start `control-plane serve` and the node-agent processes. |
| **Applies To** | `scripts/render-hosted-*.sh`, hosted control-plane proof harnesses |
| **Linked Knowledge IDs** | VGct92Y9v |
| **Score** | 0.89 |
| **Confidence** | 0.95 |
| **Applied** | yes |


# Hosted Status Contract

## Purpose

This document defines the downstream contract for the hosted K3s status payload
emitted through `port cluster status --format json`. It exists so paired infra
work can consume one Port-owned truth surface instead of reconstructing hosted
state from placement files, service probes, or ad hoc diagnostics.

## Canonical Surface

- Canonical machine-readable surface: `port cluster status --cluster <name> --format json`
- Human-readable companion surface: `port cluster status --cluster <name>`
- Scope: hosted K3s clusters only

Downstream automation should treat the JSON payload as authoritative. The text
surface mirrors the same truth for operators, but downstream consumers should
not parse the prose output.

## Stable Contract

The following fields are the stable downstream contract for hosted status
consumers.

### Top-Level Fields

| Field | Semantics | Consumer Guidance |
|------|-----------|-------------------|
| `cluster_name` | Port control-plane cluster identifier. | Stable identifier for joins across other Port surfaces. |
| `control_plane` | Hosted control-plane name backing the cluster. | Stable. |
| `host_group` | Hosted placement group used to resolve eligible nodes. | Stable. |
| `server_machines` | Declared hosted control-plane machine names. | Stable topology input. |
| `worker_machines` | Declared hosted worker machine names. | Stable topology input. |
| `api_endpoint` | Port-managed Kubernetes API endpoint exposed to consumers. | Stable endpoint consumers should prefer over guest-local kubeconfig servers. |
| `machines` | Canonical hosted machine truth entries. | Stable array; consumers may match by `machine_name`. |
| `managed_services` | Canonical hosted managed-service truth entries. | Stable array; consumers may match by `machine_name` plus `service_name`. |
| `stable_endpoint_posture` | Enum describing whether the configured `api_endpoint` is already HA-eligible or still needs manual rewrite posture. | Stable enum for gating rollout posture. |
| `ha_status` | Enum describing current hosted real-HA truth. | Stable enum for gating topology readiness. |
| `legacy_runtime_drift` | Enum describing whether legacy detached K3s runtime artifacts were detected. | Stable enum for rejecting invalid runtime shape. |
| `legacy_runtime_artifacts` | Machine/path entries describing the concrete legacy artifacts detected. | Stable when drift exists; empty when drift is clear. |
| `control_plane_placements` | Hosted placement truth for control-plane machines. | Stable array for mapping machine ownership onto hosted nodes. |

### `machines[]`

Each entry describes one hosted machine in the canonical status payload.

| Field | Semantics | Consumer Guidance |
|------|-----------|-------------------|
| `role` | Hosted machine role, currently `control-plane` or `worker`. | Stable discriminator. |
| `machine_name` | Hosted machine identifier. | Stable join key. |
| `node_name` | Selected hosted node when Port can resolve one. | Stable when present. |

Consumers should rely on `machines[]` for machine identity and selected-node
truth. A missing `node_name` means Port could not resolve a current hosted node
for that machine at the time of inspection.

### `managed_services[]`

Each entry describes the canonical K3s managed service Port expects for a
machine role.

| Field | Semantics | Consumer Guidance |
|------|-----------|-------------------|
| `role` | Hosted machine role owning the service. | Stable discriminator. |
| `machine_name` | Hosted machine identifier. | Stable join key back to `machines[]`. |
| `service_name` | Canonical Port-managed service name, currently `k3s-server` or `k3s-agent`. | Stable discriminator. |
| `state` | Managed-service truth enum: `missing`, `stored`, `starting`, `running`, `exited`, `stopped`, `failed`, or `unreachable`. | Stable gate for rollout/inspection logic. |

`managed_services[]` is intentionally canonical rather than best-effort. Port
emits one entry per expected service owner even when the service definition is
missing or the hosted node is unreachable. Consumers should reject detached or
ambiguous runtime ownership instead of treating missing service records as an
alternate happy path.

### `control_plane_placements[]`

| Field | Semantics | Consumer Guidance |
|------|-----------|-------------------|
| `machine_name` | Hosted control-plane machine identifier. | Stable join key. |
| `node_name` | Hosted node currently associated with the machine when known. | Stable when present. |

### `legacy_runtime_artifacts[]`

| Field | Semantics | Consumer Guidance |
|------|-----------|-------------------|
| `machine_name` | Hosted machine where legacy runtime drift was observed. | Stable join key. |
| `path` | Artifact path outside canonical managed-service ownership. | Stable machine-readable evidence of drift. |

## Best-Effort Diagnostics

The following fields are intentionally informative rather than stable contract
keys. Downstream consumers may display them for operator context, but should not
build gating or parsing logic that depends on their exact wording or presence.

- `stable_endpoint_detail`
- `ha_status_detail`
- `legacy_runtime_drift_detail`
- `machines[].runtime_root`
- `machines[].detail`
- `managed_services[].restart_count`
- `managed_services[].pid`
- `managed_services[].node_name`
- `managed_services[].detail`
- `legacy_runtime_artifacts[].detail`
- `control_plane_placements[].runtime_root`
- `control_plane_placements[].detail`
- `kubeconfig_surface`
- `kubeconfig`
- `visibility_surface`
- `visibility_output`
- `machine_access`
- `boundary_notes`

## Consumer Rules

1. Prefer `api_endpoint` over guest-local kubeconfig server addresses.
2. Use `ha_status`, `stable_endpoint_posture`, and `legacy_runtime_drift` for
   machine-readable gating decisions.
3. Join service truth back to `machines[]` through `machine_name`.
4. Treat `managed_services[].state=missing` or `unreachable` as degraded Port
   ownership, not as implicit permission to inspect legacy runtime artifacts.
5. Treat non-empty `legacy_runtime_artifacts[]` as invalid detached-runtime
   drift that must be resolved before downstream reuse.
6. Treat detail strings and host-local paths as operator diagnostics unless the
   field is explicitly listed in the stable contract tables above.

## Proof Posture

The hosted status contract is considered valid when these proofs continue to
pass:

| Proof Area | Command | Expected Result |
|-----------|---------|-----------------|
| Canonical typed payload | `cargo test -q -p port-runtime hosted_k3s_cluster_access_contract` | Runtime contract includes machine, placement, and managed-service truth. |
| Machine-readable JSON | `cargo test -q -p port --test machine_commands cli_cluster_status_json_surfaces_legacy_detached_runtime_drift` | CLI JSON exposes stable `machines`, `managed_services`, and drift fields. |
| Existing lifecycle/status seam | `cargo test -q -p port --test machine_commands cli_cluster_show_and_lifecycle_surface_hosted_k3s_microvms` | Hosted lifecycle continues to surface the same truth through existing CLI seams. |
| Text status projection | `cargo test -q -p port --test machine_commands cli_cluster_status_surfaces_hosted_real_ha_truth` | Human-facing status mirrors the canonical hosted truth without a separate diagnostic command. |

When running the CLI proof commands as separate processes, execute them
sequentially. The current hosted test harness shares `.port/hosted/demo` state
and is deterministic under serialized proof runs.

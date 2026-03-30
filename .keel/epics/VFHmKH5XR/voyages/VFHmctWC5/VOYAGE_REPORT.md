# VOYAGE REPORT: Replace Demo API With GitOps-Capable Local K3s Runtime

## Voyage Metadata
- **ID:** VFHmctWC5
- **Epic:** VFHmKH5XR
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Replace Demo Local Cluster Stub With Real K3s Control Plane
- **ID:** VFHn1OVki
- **Status:** done

#### Summary
Replace the shipped demo local control-plane behavior with a real single-node
K3s boot path so `port cluster up --cluster demo` brings up an actual local
cluster rather than a stub API.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `port --config examples/port.toml cluster up --cluster demo --runtime-root <tmp> --format json` succeeds on Linux and the resulting cluster runtime is backed by a real K3s control plane rather than the current demo or stub path. Verified in `EVIDENCE/ac-1.cluster-up.json`, `EVIDENCE/ac-1.console.stdout.log`, and `EVIDENCE/ac-1.firecracker-config.json`. <!-- verify: manual, SRS-01:start:end -->
- [x] [SRS-NFR-02/AC-02] The implementation keeps Port as the owner of cluster boot and readiness; no downstream `guest exec`, kubeconfig rewrite, or raw machine choreography is reintroduced as part of the fix. Verified in `EVIDENCE/ac-1.cluster-up.json` and `EVIDENCE/ac-1.cluster-down.json`, where Port owns launch, bootstrap, readiness, and teardown through the cluster surface alone. <!-- verify: manual, SRS-NFR-02:start:end -->

#### Verified Evidence
- [ac-1.cluster-down.json](../../../../stories/VFHn1OVki/EVIDENCE/ac-1.cluster-down.json)
- [ac-1.cluster-up.json](../../../../stories/VFHn1OVki/EVIDENCE/ac-1.cluster-up.json)
- [ac-1.console.stdout.log](../../../../stories/VFHn1OVki/EVIDENCE/ac-1.console.stdout.log)
- [ac-1.firecracker-config.json](../../../../stories/VFHn1OVki/EVIDENCE/ac-1.firecracker-config.json)
- [runtime-root.txt](../../../../stories/VFHn1OVki/EVIDENCE/runtime-root.txt)

### Harden Kubeconfig Handoff And Kubernetes Discovery
- **ID:** VFHn1Ozkj
- **Status:** done

#### Summary
Harden the handed-off kubeconfig and API reachability so normal Kubernetes
clients can use the Port-owned local cluster directly and discover the
resources needed for GitOps bootstrap.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] `port cluster kubeconfig --cluster demo --runtime-root <tmp> --format json` returns a kubeconfig that works with normal Kubernetes clients without downstream rewriting. Verified in `EVIDENCE/ac-1.cluster-status.json`, `EVIDENCE/ac-2.cluster-kubeconfig.json`, and `EVIDENCE/ac-2.kubectl.log`. <!-- verify: manual, SRS-02:start:end -->
- [x] [SRS-03/AC-02] `kubectl api-resources -o name` against the handed-off kubeconfig includes at least `deployments.apps`, `namespaces`, `serviceaccounts`, `secrets`, `configmaps`, and `customresourcedefinitions.apiextensions.k8s.io`. Verified in `EVIDENCE/ac-2.api-resources.log`. <!-- verify: manual, SRS-03:start:end -->

#### Verified Evidence
- [ac-1.cluster-status.json](../../../../stories/VFHn1Ozkj/EVIDENCE/ac-1.cluster-status.json)
- [ac-2.api-resources.log](../../../../stories/VFHn1Ozkj/EVIDENCE/ac-2.api-resources.log)
- [ac-2.cluster-down.json](../../../../stories/VFHn1Ozkj/EVIDENCE/ac-2.cluster-down.json)
- [ac-2.cluster-kubeconfig.json](../../../../stories/VFHn1Ozkj/EVIDENCE/ac-2.cluster-kubeconfig.json)
- [ac-2.kubectl.log](../../../../stories/VFHn1Ozkj/EVIDENCE/ac-2.kubectl.log)
- [runtime-root.txt](../../../../stories/VFHn1Ozkj/EVIDENCE/runtime-root.txt)

### Prove Flux And Pulumi Operator Install Against Port Kubeconfig
- **ID:** VFHn1PHka
- **Status:** done

#### Summary
Prove that Port's handed-off kubeconfig is GitOps-capable by running Flux and
the Pulumi Kubernetes Operator Helm install directly against the local cluster.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] `flux install` succeeds against the kubeconfig returned by `port cluster kubeconfig --cluster demo --runtime-root <tmp> --format json`. Verified in `EVIDENCE/ac-1.cluster-kubeconfig.json` and `EVIDENCE/ac-1.flux-install.log`. <!-- verify: manual, SRS-04:start:end -->
- [x] [SRS-NFR-01/AC-02] `helm upgrade --install pulumi-kubernetes-operator ...` succeeds against the same handed-off kubeconfig, and the proof records the live host-side client commands rather than only Port-local surface checks. Verified in `EVIDENCE/ac-2.helm-install.log`, `EVIDENCE/ac-2.operator-pods.log`, and `EVIDENCE/ac-2.cluster-down.json`. <!-- verify: manual, SRS-NFR-01:start:end -->

#### Verified Evidence
- [ac-0.cluster-up.json](../../../../stories/VFHn1PHka/EVIDENCE/ac-0.cluster-up.json)
- [ac-1.cluster-kubeconfig.json](../../../../stories/VFHn1PHka/EVIDENCE/ac-1.cluster-kubeconfig.json)
- [ac-1.flux-install.log](../../../../stories/VFHn1PHka/EVIDENCE/ac-1.flux-install.log)
- [ac-2.cluster-down.json](../../../../stories/VFHn1PHka/EVIDENCE/ac-2.cluster-down.json)
- [ac-2.helm-install.log](../../../../stories/VFHn1PHka/EVIDENCE/ac-2.helm-install.log)
- [ac-2.operator-pods.log](../../../../stories/VFHn1PHka/EVIDENCE/ac-2.operator-pods.log)
- [runtime-root.txt](../../../../stories/VFHn1PHka/EVIDENCE/runtime-root.txt)



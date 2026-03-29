# VOYAGE REPORT: Boot Live Local Cluster And Fix Packaged Guest Validation

## Voyage Metadata
- **ID:** VFH7t3cG9
- **Epic:** VFH7YspJx
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 4/4 stories complete

## Implementation Narrative
### Repair Local Cluster Guest Boot Path
- **ID:** VFH8C0wHN
- **Status:** done

#### Summary
Repair the shipped local guest image or boot wiring so the checked-in
single-node cluster lane boots cleanly through `/init` on Linux instead of
panicking before cluster bootstrap can begin.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `port --config examples/port.toml cluster up --cluster demo --runtime-root <tmp> --format json` succeeds on Linux without Firecracker exiting during boot or the guest failing `Run /init as init process`. <!-- [SRS-01/AC-01] verify: manual, proof: ac-1.cluster-up.json, ac-1.cluster-down.json -->

#### Verified Evidence
- [ac-1.cluster-down.json](../../../../stories/VFH8C0wHN/EVIDENCE/ac-1.cluster-down.json)
- [ac-1.cluster-up.json](../../../../stories/VFH8C0wHN/EVIDENCE/ac-1.cluster-up.json)

### Restore Live Cluster Status And Kubeconfig Handoff
- **ID:** VFH8C1KHM
- **Status:** done

#### Summary
Restore the live cluster handoff so Port reports a healthy single-node cluster
and returns a kubeconfig downstream tooling can use directly.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] `port cluster status --cluster demo --runtime-root <tmp> --format json` reports readiness=`ready`, machine_state=`running`, and kubeconfig_available=`true` after the repaired local cluster boots. Verified in `EVIDENCE/ac-1.cluster-status.json`. <!-- verify: manual, SRS-02:start:end -->
- [x] [SRS-NFR-02/AC-02] `port cluster kubeconfig --cluster demo --runtime-root <tmp> --format json` plus `kubectl get nodes -o wide` works without downstream kubeconfig rewriting or fallback `guest exec` choreography. Verified in `EVIDENCE/ac-2.cluster-kubeconfig.json`, `EVIDENCE/ac-2.kubectl.log`, and `EVIDENCE/ac-2.cluster-down.json`. <!-- verify: manual, SRS-NFR-02:start:end -->

#### Verified Evidence
- [ac-1.cluster-down.json](../../../../stories/VFH8C1KHM/EVIDENCE/ac-1.cluster-down.json)
- [ac-1.cluster-status.json](../../../../stories/VFH8C1KHM/EVIDENCE/ac-1.cluster-status.json)
- [ac-1.cluster-up.json](../../../../stories/VFH8C1KHM/EVIDENCE/ac-1.cluster-up.json)
- [ac-2.cluster-down.json](../../../../stories/VFH8C1KHM/EVIDENCE/ac-2.cluster-down.json)
- [ac-2.cluster-kubeconfig.json](../../../../stories/VFH8C1KHM/EVIDENCE/ac-2.cluster-kubeconfig.json)
- [ac-2.kubectl.log](../../../../stories/VFH8C1KHM/EVIDENCE/ac-2.kubectl.log)
- [demo.kubeconfig.yaml](../../../../stories/VFH8C1KHM/EVIDENCE/demo.kubeconfig.yaml)
- [runtime-root.txt](../../../../stories/VFH8C1KHM/EVIDENCE/runtime-root.txt)

### Fix Packaged Guest Artifact Validation Contract
- **ID:** VFH8C1fHP
- **Status:** done

#### Summary
Make the shipped guest artifact validate path install-safe so `port artifacts
validate` works from the packaged CLI contract instead of resolving scripts
from source-build-only locations.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] `port --config examples/port.toml artifacts validate --artifact demo-guest --architecture x86-64` succeeds without looking for `validate-guest-image.sh` under `/build/...`. Verified in `EVIDENCE/ac-1.log`, `EVIDENCE/ac-1.nix-package-validate.log`, `EVIDENCE/ac-2.package-proof.log`, and `EVIDENCE/ac-3.prefix-validate.log`. <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->

#### Verified Evidence
- [absolute-artifact-config.toml](../../../../stories/VFH8C1fHP/EVIDENCE/absolute-artifact-config.toml)
- [ac-1.log](../../../../stories/VFH8C1fHP/EVIDENCE/ac-1.log)
- [ac-1.nix-package-validate.log](../../../../stories/VFH8C1fHP/EVIDENCE/ac-1.nix-package-validate.log)
- [ac-2.package-proof.log](../../../../stories/VFH8C1fHP/EVIDENCE/ac-2.package-proof.log)
- [ac-3.prefix-validate.log](../../../../stories/VFH8C1fHP/EVIDENCE/ac-3.prefix-validate.log)

### Verify Downstream Local Cluster Handoff
- **ID:** VFH8C1xHO
- **Status:** done

#### Summary
Verify that the repaired local cluster lane is actually consumable by downstream
tooling and that the mission stays bounded to the intended single-node local
runtime slice.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] Downstream verification shows `spoke infra` can treat Port as the owner of cluster handoff readiness without reviving AWS, hosted cluster, or multi-node work in this mission. Verified in `EVIDENCE/ac-1.infra-bootstrap.log` and `EVIDENCE/ac-0.infra-proof-meta.txt`. <!-- verify: manual, SRS-04:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-01/AC-02] Story evidence includes live local cluster boot proof, packaged artifact validate proof, and one downstream handoff check rather than proof-only surface artifacts. Verified in `../VFH8C0wHN/EVIDENCE/ac-1.cluster-up.json`, `../VFH8C1fHP/EVIDENCE/ac-1.log`, and `EVIDENCE/ac-1.infra-bootstrap.log`. <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-2.log-->
- [x] [SRS-NFR-03/AC-03] The final mission slice keeps explicit single-node local boundaries and leaves AWS, hosted cluster, and multi-node expansion as follow-on work. Verified in `EVIDENCE/ac-0.infra-proof-meta.txt`, `EVIDENCE/ac-3.port-cluster-down.json`, and the voyage SRS/SDD scope boundaries. <!-- verify: manual, SRS-NFR-03:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-0.infra-proof-meta.txt](../../../../stories/VFH8C1xHO/EVIDENCE/ac-0.infra-proof-meta.txt)
- [ac-1.infra-bootstrap.log](../../../../stories/VFH8C1xHO/EVIDENCE/ac-1.infra-bootstrap.log)
- [ac-1.log](../../../../stories/VFH8C1xHO/EVIDENCE/ac-1.log)
- [ac-2.infra-ps.log](../../../../stories/VFH8C1xHO/EVIDENCE/ac-2.infra-ps.log)
- [ac-2.log](../../../../stories/VFH8C1xHO/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VFH8C1xHO/EVIDENCE/ac-3.log)
- [ac-3.port-cluster-down.json](../../../../stories/VFH8C1xHO/EVIDENCE/ac-3.port-cluster-down.json)



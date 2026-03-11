# VOYAGE REPORT: Service Policy And Secret Runtime Foundations

## Voyage Metadata
- **ID:** 1vzfTm000
- **Epic:** 1vzfT4000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 4/4 stories complete

## Implementation Narrative
### Implement Service Supervision And Health State
- **ID:** 1vzfV4000
- **Status:** done

#### Summary
Extend Port's runtime owner into a real managed-process supervisor that
enforces restart policy, tracks restart and exit state, and reports service
health through the canonical local and hosted status surfaces.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] Port supervises managed service or sandbox processes according to the selected restart policy and records restart count, last exit detail, and health state under the existing runtime owner. <!-- [SRS-02/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime service_supervision && cargo test -q -p port-runtime service_health', proof: ac-1.log -->
- [x] [SRS-02/AC-02] Local and hosted `port service status` project the same restart and health state without introducing a second service runtime model. <!-- [SRS-02/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-cli service_status && cargo test -q -p port-sdk service_status', proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzfV4000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzfV4000/EVIDENCE/ac-2.log)

### Publish Service Reliability Operator Workflow
- **ID:** 1vzfV5000
- **Status:** done

#### Summary
Publish the shipped service-reliability workflow across the CLI, docs, sample
config, and recorded proofs so operators can discover and execute secret-backed
services with restart and health visibility through canonical Port commands.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] README, hosted/operator docs, CLI help, and sample-config guidance publish the service reliability workflow and its remaining limits through the canonical `port service` surface. <!-- [SRS-04/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && bash scripts/verify-service-reliability-docs.sh', proof: ac-1.log -->
- [x] [SRS-04/AC-02] Port records a repo-local proof that stores a secret, launches a service, observes health or restart state, and stops the workload through canonical `port service` verbs. <!-- [SRS-04/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && bash scripts/service-reliability-demo.sh', proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzfV5000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzfV5000/EVIDENCE/ac-2.log)

### Define Service Policy And Health Contract
- **ID:** 1vzfVW000
- **Status:** done

#### Summary
Define the shared restart-policy and health-policy contract for Port services
and sandboxes, then thread that contract through the canonical CLI, hosted API,
and SDK surfaces without introducing a second service model.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `port service` config, help, hosted request/status payloads, and SDK types expose the shared restart-policy and health-policy contract without adding a hosted-only alias. <!-- [SRS-01/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-model service_policy && cargo test -q -p port-sdk service_policy && cargo test -q -p port-cli service_policy', proof: ac-1.log -->
- [x] [SRS-01/AC-02] Unsupported restart-policy or health-policy combinations fail fast with explicit diagnostics and no fallback to legacy service behavior, satisfying `SRS-NFR-02`. <!-- [SRS-01/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-model service_policy_invalid && cargo test -q -p port-cli service_policy_invalid', proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzfVW000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzfVW000/EVIDENCE/ac-2.log)

### Implement Secret Backend And Materialization
- **ID:** 1vzfVh000
- **Status:** done

#### Summary
Replace plaintext runtime JSON as the canonical service-execution secret input
with a stronger runtime-owned backend plus explicit materialization behavior
for `port service secret` and `port service apply`.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] `port service secret put|list|remove` and service launch resolution use the new secret-backend and materialization contract rather than legacy JSON-secret execution. <!-- [SRS-03/AC-01] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime service_secret_backend && cargo test -q -p port-cli service_secret_backend', proof: ac-1.log -->
- [x] [SRS-03/AC-02] Service status surfaces secret-source provenance and materialization detail without leaking secret contents and keeps that state attributable to one runtime owner, satisfying `SRS-NFR-01`. <!-- [SRS-03/AC-02] verify: bash -lc 'cd /home/alex/workspace/spoke-sh/port && cargo test -q -p port-runtime service_secret_status && cargo test -q -p port-cli service_secret_status', proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzfVh000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzfVh000/EVIDENCE/ac-2.log)



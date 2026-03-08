# VOYAGE REPORT: Executable Avf Runtime Foundation

## Voyage Metadata
- **ID:** 1vzLYD000
- **Epic:** 1vzJKE000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 4/4 stories complete

## Implementation Narrative
### Define Avf Machine Contract And Doctor Checks
- **ID:** 1vzLZS000
- **Status:** done

#### Summary
Define the macOS-only AVF machine-selection and doctor contract so Port can
identify valid AVF targets, reject unsupported hosts, and surface entitlement
or availability boundaries before runtime work lands.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] AVF-targeted machines validate as macOS-only `standard`-protection local machines and fail fast on non-macOS or AVF/PVM selections. <!-- [SRS-01/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzLZS000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-02/AC-02] `port doctor` surfaces AVF-focused macOS checks plus explicit AVF availability or entitlement boundaries through the canonical CLI output. <!-- [SRS-02/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzLZS000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzLZS000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzLZS000/EVIDENCE/ac-2.log)

### Publish Macos Avf Operator Workflow
- **ID:** 1vzLZY000
- **Status:** done

#### Summary
Publish the native macOS AVF workflow across the CLI help and docs once the
runtime slices are in place, including proof commands and explicit unsupported
boundaries.

#### Acceptance Criteria
- [x] [SRS-05/AC-01] CLI help, README, `docs/avf.md`, and macOS operator docs describe the native AVF workflow, prerequisites, and unsupported boundaries through the canonical `port` command model. <!-- [SRS-05/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzLZY000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-05/AC-02] Recorded proof demonstrates the AVF workflow contract through the canonical CLI and docs surfaces for a new operator. <!-- [SRS-05/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzLZY000/verify-ac-2.sh, proof: ac-2.log -->
- [x] [SRS-06/AC-01] Recorded proof demonstrates the AVF workflow contract while also preserving explicit Linux-lane and unsupported-host boundaries for operators. <!-- [SRS-06/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzLZY000/verify-ac-3.sh, proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzLZY000/EVIDENCE/ac-1.log)
- [ac-3.log](../../../../stories/1vzLZY000/EVIDENCE/ac-3.log)
- [ac-2.log](../../../../stories/1vzLZY000/EVIDENCE/ac-2.log)

### Implement Avf Local Machine Driver
- **ID:** 1vzLZj000
- **Status:** done

#### Summary
Add the first AVF local machine driver behind the shared runtime seam so
`machine launch`, `status`, and `stop` can own AVF-backed VMs without
introducing a substrate-specific command family.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] `port machine launch`, `status`, and `stop` route AVF-targeted machines through a local AVF driver that writes canonical runtime manifests plus AVF-specific runtime metadata. <!-- [SRS-03/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzLZj000/verify-ac-1.sh, proof: ac-1.log -->
- [x] [SRS-03/AC-02] The AVF local driver keeps deterministic runtime metadata and explicit substrate-specific failure detail instead of falling back silently to existing Linux lanes. <!-- [SRS-03/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzLZj000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzLZj000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzLZj000/EVIDENCE/ac-2.log)

### Wire Avf Guest Transport And Console Capture
- **ID:** 1vzLZm000
- **Status:** done

#### Summary
Map the shared guest protocol onto the AVF transport and serial-console
surfaces so the canonical `guest` verbs and machine log inspection work for
AVF-backed machines.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] AVF-targeted machines expose `guest exec|copy|pty|logs|forward` through the canonical CLI and shared guest protocol via an AVF transport adapter. <!-- [SRS-04/AC-01] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzLZm000/verify-ac-1.sh, proof: ac-2.log -->
- [x] [SRS-04/AC-02] AVF boot and console output land in canonical runtime log surfaces that `machine status` and operator inspection can reference. <!-- [SRS-04/AC-02] verify: bash /home/alex/workspace/spoke-sh/port/.keel/stories/1vzLZm000/verify-ac-2.sh, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzLZm000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzLZm000/EVIDENCE/ac-2.log)



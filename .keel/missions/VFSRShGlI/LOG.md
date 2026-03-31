# Enable Guest VM Outbound Networking - Decision Log

<!-- Append entries below. Each entry is an H2 with ISO timestamp. -->
<!-- Use `keel mission digest` to compress older entries when this file grows large. -->

## 2026-03-31T08:07:08

Created mission VFSRShGlI to enable guest VM outbound networking for Firecracker VMs. The mission addresses the missing network interface that blocks Flux GitOps reconciliation in spoke-sh/infra — without TAP/virtio-net and host-side NAT, CoreDNS cannot resolve, containerd cannot pull images, and Flux cannot clone from GitHub, leaving the cluster stuck at 5/18 components. Three goals defined: MG-01 guest outbound networking via TAP + NAT, MG-02 guest DNS resolution, MG-03 configurable service forwards beyond the API tunnel.

## 2026-03-31T08:33:46

Activated mission VFSRShGlI. Created epic VFSWpHXG1 (Guest VM Outbound Networking) with authored PRD, planned voyage VFSXWpO18 (TAP Networking and Host NAT for Local Firecracker VMs) with SRS and SDD, and decomposed three execution stories: VFSXxsKe2 (TAP/virtio-net + host NAT), VFSXyBMiG (guest DNS + eth0 bring-up), VFSXySLmb (configurable port forwards). All stories are in backlog ready for execution.

# Enable Guest VM Outbound Networking - Decision Log

<!-- Append entries below. Each entry is an H2 with ISO timestamp. -->
<!-- Use `keel mission digest` to compress older entries when this file grows large. -->

## 2026-03-31T08:07:08

Created mission VFSRShGlI to enable guest VM outbound networking for Firecracker VMs. The mission addresses the missing network interface that blocks Flux GitOps reconciliation in spoke-sh/infra — without TAP/virtio-net and host-side NAT, CoreDNS cannot resolve, containerd cannot pull images, and Flux cannot clone from GitHub, leaving the cluster stuck at 5/18 components. Three goals defined: MG-01 guest outbound networking via TAP + NAT, MG-02 guest DNS resolution, MG-03 configurable service forwards beyond the API tunnel.

## 2026-03-31T08:33:46

Activated mission VFSRShGlI. Created epic VFSWpHXG1 (Guest VM Outbound Networking) with authored PRD, planned voyage VFSXWpO18 (TAP Networking and Host NAT for Local Firecracker VMs) with SRS and SDD, and decomposed three execution stories: VFSXxsKe2 (TAP/virtio-net + host NAT), VFSXyBMiG (guest DNS + eth0 bring-up), VFSXySLmb (configurable port forwards). All stories are in backlog ready for execution.

## 2026-03-31T10:06:22

Executed all three stories for voyage VFSXWpO18. Implementation spans five files across model, runtime, guest image, and CLI layers:

1. port-model: Added MachineNetworkSpec (enabled, guest_ip, host_ip, prefix_len, guest_mac, dns_servers) and ServiceForwardSpec. Extended MachineSpec with optional network config and ClusterLifecycleSpec with forwards vec.

2. port-runtime: Added NetworkInterfaceConfig to FirecrackerConfig JSON output. Added setup_host_networking() (TAP device, iptables MASQUERADE, FORWARD rules) called before VM boot and teardown_host_networking() called on VM stop. Network state persisted to runtime dir for orphan cleanup. Boot args extended with port.net_ip, port.net_gateway, port.net_prefix_len, port.net_dns kernel cmdline params.

3. Guest image: Added virtio_net module to initrd init. Added ip busybox applet. Guest init now parses network params from /proc/cmdline, brings up eth0 with static IP, adds default route, and generates /etc/resolv.conf from DNS servers.

4. port-cli: cluster up now establishes additional forwards from lifecycle config. cluster down tears them down.

5. examples/port.toml: Added [machines.demo.network] section and [[clusters.demo.lifecycle.forwards]] entries.

All 54 port-model tests pass. 130/132 port-runtime tests pass (2 pre-existing failures). cargo build succeeds. Voyage VFSXWpO18 and epic VFSWpHXG1 auto-completed.

## 2026-03-31T10:06:27

Mission achieved by local system user 'alex'

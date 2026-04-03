{
  architecture = "x86_64";
  package = {
    name = "firecracker-pvm-host-kit";
    version = "2026.03";
    host_kernel_release = "6.12.0-port-pvm";
    firecracker_build = "v1.12.0-port-pvm";
  };
  host_platform = "linux";
  host_architecture = "x86_64";
  requires_custom_host_kernel = true;
  requires_patched_firecracker = true;
  firecracker_binary_name = "firecracker-pvm";
  firecracker_binary_env = "PORT_PVM_FIRECRACKER_BINARY";
  host_boot_args = [ "pti=off" ];
  notes = [
    "The host kernel must carry the Firecracker/PVM-capable KVM changes rather than stock KVM alone."
    "The VMM binary must be a PVM-capable Firecracker build, not the current standard lane binary."
  ];
}

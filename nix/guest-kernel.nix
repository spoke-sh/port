{ lib, linuxPackages_6_12, linuxPackagesFor }:

# Port-owned guest kernel for the standard x86_64/firecracker lane.
#
# Pinned to the 6.12 LTS series (kernel 7.x does not yet boot under PVM)
# and overridden with `CONFIG_DUMMY=y` so the guest's K3s install proof
# can `ip link add port0 type dummy` without needing the dummy.ko module
# on the rootfs. The result is a full `linuxPackages` set so callers
# retain `.kernel.dev` and `.kernel.modules`.
linuxPackagesFor (linuxPackages_6_12.kernel.override {
  structuredExtraConfig = with lib.kernel; {
    DUMMY = yes;
  };
})

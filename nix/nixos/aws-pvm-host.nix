{
  config,
  lib,
  pkgs,
  ...
}:
let
  hostKit = import ./aws-pvm-host-kit-contract.nix;
  cfg = config.port.awsPvmHost;
  fallbackKernelPackages = pkgs.linuxPackages_6_12;
  defaultFirecrackerPackage = pkgs.callPackage ../loopholelabs-firecracker-pvm.nix { };
  hostKitPackage = pkgs.callPackage ../firecracker-pvm-host-kit.nix {
    firecracker = cfg.firecrackerPackage;
  };
  hostKitManifestPath = "/etc/port/aws-pvm-host-kit.json";
  firecrackerBinaryPath = "/run/current-system/sw/bin/${hostKit.firecracker_binary_name}";
in
{
  options.port.awsPvmHost = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Whether to apply Port's AWS x86_64 PVM host-kit contract.";
    };

    kernelPackages = lib.mkOption {
      type = lib.types.raw;
      default = fallbackKernelPackages;
      description = ''
        Kernel package set used for the AWS PVM host contract. The default is a
        buildable Linux 6.12 fallback so downstream NixOS image builds can
        evaluate and realize cleanly; override this with the concrete
        PVM-capable kernel derivation used by your image pipeline.
      '';
    };

    firecrackerPackage = lib.mkOption {
      type = lib.types.package;
      default = defaultFirecrackerPackage;
      description = ''
        Pinned loopholelabs Firecracker/PVM derivation wrapped as the canonical
        `firecracker-pvm` binary surface. Override this only when the image
        pipeline intentionally ships a different provenanced PVM build.
      '';
    };

    package = lib.mkOption {
      type = lib.types.package;
      readOnly = true;
      description = ''
        Resolved Port AWS PVM host-kit package that publishes the canonical
        `firecracker-pvm` wrapper, packaged module path, and host-kit manifest.
      '';
    };

    hostKit = lib.mkOption {
      type = lib.types.attrs;
      readOnly = true;
      description = ''
        Canonical AWS PVM host-kit metadata exported by Port and aligned with
        the `prepare-pvm-node` contract.
      '';
    };

    manifestPath = lib.mkOption {
      type = lib.types.str;
      readOnly = true;
      description = "Path to the installed AWS PVM host-kit manifest on the host.";
    };
  };

  config = lib.mkIf cfg.enable {
    assertions = [
      {
        assertion = pkgs.stdenv.hostPlatform.isLinux;
        message = "Port's aws-pvm-host module only supports Linux hosts.";
      }
    ];

    port.awsPvmHost.package = hostKitPackage;
    port.awsPvmHost.hostKit = hostKit;
    port.awsPvmHost.manifestPath = hostKitManifestPath;

    boot.kernelPackages = lib.mkDefault cfg.kernelPackages;
    boot.kernelModules = [ "kvm" "tun" "bridge" ];
    boot.kernelParams = lib.mkAfter hostKit.host_boot_args;

    environment.systemPackages = [
      hostKitPackage
      pkgs.bridge-utils
      pkgs.iproute2
    ];

    environment.variables.${hostKit.firecracker_binary_env} = lib.mkDefault firecrackerBinaryPath;

    environment.etc."port/aws-pvm-host-kit.json".text = builtins.toJSON hostKit;
  };
}

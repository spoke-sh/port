{
  description = "Port - Agentic compute orchestration in Firecracker VMs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/c23fa0ff46bbbfc4026e2f54cf4facf713dae7da";
    k3s-nixpkgs.url = "github:NixOS/nixpkgs/173d0ad7a974f8543a9ab01d2271b2e290341b33";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    sift = {
      url = "github:rupurt/sift?ref=main";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-overlay.follows = "rust-overlay";
      inputs.flake-utils.follows = "flake-utils";
    };
    atxt = {
      url = "github:spoke-sh/atxt?ref=main";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-overlay.follows = "rust-overlay";
      inputs.flake-utils.follows = "flake-utils";
      inputs.keel.follows = "keel";
      inputs.sift.follows = "sift";
    };
    paddles = {
      url = "github:spoke-sh/paddles?ref=main";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-overlay.follows = "rust-overlay";
      inputs.flake-utils.follows = "flake-utils";
      inputs.keel.follows = "keel";
      inputs.sift.follows = "sift";
    };
    keel = {
      url = "github:spoke-sh/keel?ref=main";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-overlay.follows = "rust-overlay";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = {
    self,
    nixpkgs,
    k3s-nixpkgs,
    rust-overlay,
    flake-utils,
    sift,
    atxt,
    paddles,
    keel,
  }:
    let
      awsPvmHostKit = import ./nix/nixos/aws-pvm-host-kit-contract.nix;
      exampleConfig = builtins.fromTOML (builtins.readFile ./examples/port.toml);
      exampleAwsPvmLane =
        let
          matches = builtins.filter
            (lane: lane.architecture == awsPvmHostKit.architecture)
            exampleConfig.nodes."aws-linux-node".capabilities.pvm_lanes;
        in
          if matches == [ ] then
            throw "examples/port.toml does not define the canonical aws-linux-node x86_64 PVM lane"
          else
            builtins.head matches;
      exampleAwsPvmHostKit = exampleAwsPvmLane.host_kit;
      _awsPvmHostKitSync = assert awsPvmHostKit == exampleAwsPvmHostKit; true;
      awsPvmHostModule = import ./nix/nixos/aws-pvm-host.nix;
    in
      flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        k3sPkgs = import k3s-nixpkgs { inherit system; };
        k3sPkg = k3sPkgs.k3s_1_35;
        rust = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "llvm-tools" ];
        };
        isLinux = pkgs.stdenv.isLinux;
        isDarwin = pkgs.stdenv.isDarwin;
        siftPkg = sift.packages.${system}.sift;
        portUnwrapped = pkgs.callPackage ./nix/port.nix {
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rust;
            rustc = rust;
          };
        };
        portRuntimeDeps = [
          pkgs.oras
          pkgs.gnutar
          pkgs.gzip
          pkgs.curl
        ] ++ pkgs.lib.optionals isLinux [
          k3sPkg
          awsPvmHostKitPkg
          pkgs.firecracker
          pkgs.iproute2
          pkgs.iptables
          pkgs.busybox
          pkgs.cpio
          pkgs.e2fsprogs
        ];
        portPkg = pkgs.symlinkJoin {
          name = "port-${portUnwrapped.version}";
          paths = [ portUnwrapped ];
          nativeBuildInputs = [ pkgs.makeWrapper ];
          postBuild = ''
            wrapProgram $out/bin/port \
              --prefix PATH : ${pkgs.lib.makeBinPath portRuntimeDeps}
          '';
        };
        atxtPkg = pkgs.callPackage (
          {
            lib,
            rustPlatform,
            ...
          }:
            let
              cargoToml = lib.importTOML "${atxt}/Cargo.toml";
            in
              rustPlatform.buildRustPackage {
                pname = cargoToml.package.name;
                version = cargoToml.package.version;

                src = atxt;

                doCheck = false;

                cargoLock = {
                  lockFile = "${atxt}/Cargo.lock";
                  outputHashes = {
                    "txtplot-0.1.0" = "sha256-bC6zo1yhJg41iz69XbXqwIKOfNVXwFke0vzcSMbqvFE=";
                  };
                };

                meta = with lib; {
                  description = cargoToml.package.description;
                  homepage = "https://github.com/spoke-sh/atxt";
                  license = licenses.mit;
                  maintainers = [ ];
                };
              }
        ) {
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rust;
            rustc = rust;
          };
        };
        atxtAliasPkg = pkgs.writeShellScriptBin "atxt" ''
          exec ${atxtPkg}/bin/atext "$@"
        '';
        paddlesPkg = paddles.packages.${system}.paddles;
        keelPkg = keel.packages.${system}.keel;
        awsPvmFirecrackerPkg = pkgs.callPackage ./nix/loopholelabs-firecracker-pvm.nix { };
        awsPvmHostKitPkg = pkgs.callPackage ./nix/firecracker-pvm-host-kit.nix {
          firecracker = awsPvmFirecrackerPkg;
        };
        awsPvmHostModuleEval = nixpkgs.lib.nixosSystem {
          inherit system;
          modules = [
            awsPvmHostModule
            {
              system.stateVersion = "24.11";
            }
          ];
        };
        _awsPvmHostKitPkgAssertions = assert awsPvmHostKitPkg.passthru.hostKit == awsPvmHostKit; true;
        _awsPvmHostKitPkgAssertions2 =
          assert awsPvmHostKitPkg.passthru.modulePath
            == "/share/port/nixos/aws-pvm-host.nix"; true;
        _awsPvmHostKitPkgAssertions3 =
          assert awsPvmHostKitPkg.passthru.manifestPath
            == "/share/port/aws-pvm-host-kit.json"; true;
        _awsPvmHostModuleAssertions = assert awsPvmHostModuleEval.config.port.awsPvmHost.hostKit == awsPvmHostKit; true;
        _awsPvmHostModuleAssertions2 = assert builtins.elem "pti=off" awsPvmHostModuleEval.config.boot.kernelParams; true;
        _awsPvmHostModuleAssertions3 =
          assert awsPvmHostModuleEval.config.environment.variables.PORT_PVM_FIRECRACKER_BINARY
            == "/run/current-system/sw/bin/firecracker-pvm"; true;
        _awsPvmHostModuleAssertions4 =
          assert builtins.any
            (pkg: pkg == awsPvmHostModuleEval.config.port.awsPvmHost.package)
            awsPvmHostModuleEval.config.environment.systemPackages; true;
        _awsPvmHostModuleAssertions5 =
          assert awsPvmHostModuleEval.config.boot.kernelPackages.kernel.modDirVersion
            == awsPvmHostModuleEval.config.boot.kernelPackages.kernel.version; true;
        _ = [
          _awsPvmHostKitSync
          _awsPvmHostKitPkgAssertions
          _awsPvmHostKitPkgAssertions2
          _awsPvmHostKitPkgAssertions3
          _awsPvmHostModuleAssertions
          _awsPvmHostModuleAssertions2
          _awsPvmHostModuleAssertions3
          _awsPvmHostModuleAssertions4
          _awsPvmHostModuleAssertions5
        ];
        sharedInputs = [
          rust
          portPkg
          pkgs.nodejs_24
          pkgs.kubernetes-helm
          pkgs.fluxcd
          pkgs.just
          pkgs.less
          pkgs.gnutar
          pkgs.gzip
          pkgs.vhs
          pkgs.oras
          pkgs.cargo-nextest
          pkgs.cargo-llvm-cov
          siftPkg
          atxtAliasPkg
          atxtPkg
          paddlesPkg
          keelPkg
          pkgs.curl
        ] ++ pkgs.lib.optionals isLinux [
          k3sPkg
        ];
        linuxRuntimeInputs = pkgs.lib.optionals isLinux [
          pkgs.firecracker
          awsPvmHostKitPkg
          pkgs.iproute2
          pkgs.iptables
          pkgs.cpio
          pkgs.e2fsprogs
          pkgs.mold
          pkgs.skopeo
        ];
      in {
        packages = {
          port = portPkg;
          atext = atxtPkg;
          atxt = atxtAliasPkg;
          keel = keelPkg;
          default = portPkg;
        } // pkgs.lib.optionalAttrs isLinux {
          k3s = k3sPkg;
          k3s-airgap-images = k3sPkg.airgap-images;
          firecracker-pvm = awsPvmFirecrackerPkg;
          firecracker-pvm-host-kit = awsPvmHostKitPkg;
        };

        legacyPackages = pkgs.lib.optionalAttrs isLinux {
          linuxPackages-port-guest = pkgs.callPackage ./nix/guest-kernel.nix { };
        };

        checks = pkgs.lib.optionalAttrs isLinux {
          aws-pvm-host-module-eval = pkgs.writeText "aws-pvm-host-module-eval.json" (
            builtins.toJSON {
              module = "aws-pvm-host";
              bootKernelParams = awsPvmHostModuleEval.config.boot.kernelParams;
              firecrackerBinary = awsPvmHostModuleEval.config.environment.variables.PORT_PVM_FIRECRACKER_BINARY;
              manifestPath = awsPvmHostModuleEval.config.port.awsPvmHost.manifestPath;
              hostKit = awsPvmHostModuleEval.config.port.awsPvmHost.hostKit;
              hostKitPackage = awsPvmHostModuleEval.config.port.awsPvmHost.package.pname
                or awsPvmHostModuleEval.config.port.awsPvmHost.package.name;
              kernelVersion = awsPvmHostModuleEval.config.boot.kernelPackages.kernel.version;
              kernelModDirVersion = awsPvmHostModuleEval.config.boot.kernelPackages.kernel.modDirVersion;
              modulePath = "${awsPvmHostKitPkg}${awsPvmHostKitPkg.passthru.modulePath}";
            }
          );
        };

        devShells.default = pkgs.mkShell {
          buildInputs = sharedInputs ++ linuxRuntimeInputs;

          shellHook = ''
            export CARGO_TARGET_DIR="$HOME/.cache/cargo-target/port"
          '' + pkgs.lib.optionalString isDarwin ''
            export TMPDIR=/var/tmp
            echo "Port's macOS dev shell provides repo tooling only; Linux-only runtime tools such as firecracker, iproute2, and iptables remain available only on Linux hosts."
          '' + pkgs.lib.optionalString isLinux ''
            export PORT_BUSYBOX_BIN="${pkgs.busybox}/bin/busybox"
            export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"
            export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"
          '';
        };
      })
      // {
        lib = {
          awsPvmHostKit = awsPvmHostKit;
        };

        nixosModules = {
          aws-pvm-host = awsPvmHostModule;
          default = awsPvmHostModule;
        };
      };
}

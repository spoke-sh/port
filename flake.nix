{
  description = "Port - Agentic compute orchestration in Firecracker VMs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    sift = {
      url = "github:rupurt/sift?ref=main";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-overlay.follows = "rust-overlay";
      inputs.flake-utils.follows = "flake-utils";
    };
    atxt = {
      url = "git+ssh://git@github.com/spoke-sh/atxt.git?ref=main";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-overlay.follows = "rust-overlay";
      inputs.flake-utils.follows = "flake-utils";
      inputs.keel.follows = "keel";
      inputs.sift.follows = "sift";
    };
    keel = {
      url = "git+ssh://git@github.com/spoke-sh/keel.git?ref=main";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-overlay.follows = "rust-overlay";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    sift,
    atxt,
    keel,
  }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
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
          pkgs.k3s
          pkgs.oras
          pkgs.gnutar
          pkgs.gzip
          pkgs.curl
        ] ++ pkgs.lib.optionals isLinux [
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
                    "txtplot-0.1.0" = "sha256-XPDnH8Bo461tdizRS00P3A7eg+yEgUyKIls7W/OHCt4=";
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
        keelPkg = keel.packages.${system}.keel;
        sharedInputs = [
          rust
          portPkg
          pkgs.nodejs_22
          pkgs.k3s
          pkgs.kubernetes-helm
          pkgs.fluxcd
          pkgs.just
          pkgs.gnutar
          pkgs.gzip
          pkgs.vhs
          pkgs.oras
          pkgs.cargo-nextest
          pkgs.cargo-llvm-cov
          siftPkg
          atxtAliasPkg
          atxtPkg
          keelPkg
          pkgs.curl
        ];
        linuxRuntimeInputs = pkgs.lib.optionals isLinux [
          pkgs.firecracker
          pkgs.iproute2
          pkgs.iptables
          pkgs.busybox
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
        };

        devShells.default = pkgs.mkShell {
          buildInputs = sharedInputs ++ linuxRuntimeInputs;

          shellHook = ''
            export CARGO_TARGET_DIR="$HOME/.cache/cargo-target/port"
          '' + pkgs.lib.optionalString isDarwin ''
            export TMPDIR=/var/tmp
            echo "Port's macOS dev shell provides repo tooling only; Linux-only runtime tools such as firecracker, iproute2, and iptables remain available only on Linux hosts."
          '' + pkgs.lib.optionalString isLinux ''
            export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"
            export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"
          '';
        };
      });
}

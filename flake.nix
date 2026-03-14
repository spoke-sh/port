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
    keel = {
      url = "git+ssh://git@github.com/spoke-sh/keel.git?ref=refs/heads/main";
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
        keelPkg = pkgs.callPackage (
          {
            lib,
            rustPlatform,
            pkg-config,
            zstd,
            git,
            ...
          }:
            let
              cargoToml = lib.importTOML "${keel}/Cargo.toml";
            in
              rustPlatform.buildRustPackage {
                pname = "keel";
                version = cargoToml.package.version;

                src = keel;

                doCheck = false;

                cargoLock = {
                  lockFile = "${keel}/Cargo.lock";
                  outputHashes = {
                    "txtplot-0.1.0" = "sha256-bC6zo1yhJg41iz69XbXqwIKOfNVXwFke0vzcSMbqvFE=";
                  };
                };

                nativeBuildInputs = [
                  pkg-config
                ];

                nativeCheckInputs = [
                  git
                ];

                buildInputs = [
                  zstd
                ];

                meta = with lib; {
                  description = "Fast CLI for project board management";
                  homepage = "https://github.com/spoke-sh/keel";
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
        sharedInputs = [
          rust
          pkgs.just
          pkgs.gnutar
          pkgs.gzip
          pkgs.vhs
          pkgs.oras
          pkgs.cargo-nextest
          pkgs.cargo-llvm-cov
          siftPkg
          keelPkg
          pkgs.curl
        ];
        linuxRuntimeInputs = pkgs.lib.optionals isLinux [
          pkgs.firecracker
          pkgs.iproute2
          pkgs.iptables
          pkgs.busybox
          pkgs.e2fsprogs
          pkgs.mold
        ];
      in {
        packages = {
          keel = keelPkg;
          default = keelPkg;
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

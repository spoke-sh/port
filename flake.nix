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
      url = "github:spoke-sh/keel?ref=main";
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
        keelPkg = keel.packages.${system}.keel;
        sharedInputs = [
          rust
          pkgs.just
          pkgs.debianutils
          pkgs.procps
          pkgs.gnutar
          pkgs.gzip
          pkgs.chromium
          pkgs.playwright-driver.browsers
          pkgs.ttyd
          pkgs.ffmpeg
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
            export PATH="$PWD/scripts/bin:$PATH"
            export PORT_HEADLESS_SHELL="$(find ${pkgs.playwright-driver.browsers} -path '*chrome-headless-shell-linux64/chrome-headless-shell' -print -quit 2>/dev/null)"
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

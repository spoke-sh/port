{
  description = "Port - Agentic compute orchestration in Firecracker VMs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    keel = {
      url = "git+ssh://git@github.com/rupurt/keel.git";
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
        keelPkg = keel.packages.${system}.keel;
      in {
        packages = {
          keel = keelPkg;
          default = keelPkg;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            rust
            pkgs.just
            pkgs.cargo-nextest
            keelPkg
            pkgs.firecracker
            pkgs.iproute2
            pkgs.iptables
            pkgs.busybox
            pkgs.curl
            pkgs.e2fsprogs
          ] ++ pkgs.lib.optionals isLinux [
            pkgs.mold
          ];

          shellHook = ''
            export CARGO_TARGET_DIR="$HOME/.cache/cargo-target/port"
          '' + pkgs.lib.optionalString isDarwin ''
            export TMPDIR=/var/tmp
          '' + pkgs.lib.optionalString isLinux ''
            export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"
            export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"
          '';
        };
      });
}

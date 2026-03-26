{ lib, rustPlatform, ... }:

let
  workspaceToml = lib.importTOML ../Cargo.toml;
in
rustPlatform.buildRustPackage {
  pname = "port";
  version = workspaceToml.workspace.package.version;

  src = ../.;

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  cargoBuildFlags = [
    "--package"
    "port-cli"
    "--bin"
    "port"
  ];

  doCheck = false;

  meta = with lib; {
    description = "Agentic compute orchestration CLI";
    homepage = "https://github.com/spoke-sh/port";
    license = licenses.mit;
    maintainers = [ ];
    mainProgram = "port";
  };
}

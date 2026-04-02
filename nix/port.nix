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
    "port"
    "--bin"
    "port"
  ];

  doCheck = false;

  postInstall = ''
    install -d $out/share/port/examples/bootstrap/demo-k3s
    install -Dm755 ${../scripts/artifacts/build-kernel.sh} \
      $out/share/port/scripts/artifacts/build-kernel.sh
    install -Dm755 ${../examples/bootstrap/demo-k3s/install-k3s-offline.sh} \
      $out/share/port/examples/bootstrap/demo-k3s/install-k3s-offline.sh
    install -Dm755 ${../examples/bootstrap/demo-k3s/k3s} \
      $out/share/port/examples/bootstrap/demo-k3s/k3s
    install -Dm644 ${../examples/bootstrap/demo-k3s/preloaded-images.txt} \
      $out/share/port/examples/bootstrap/demo-k3s/preloaded-images.txt
    install -Dm755 ${../scripts/artifacts/build-guest-image.sh} \
      $out/share/port/scripts/artifacts/build-guest-image.sh
    install -Dm755 ${../scripts/artifacts/validate-kernel.sh} \
      $out/share/port/scripts/artifacts/validate-kernel.sh
    install -Dm755 ${../scripts/artifacts/validate-guest-image.sh} \
      $out/share/port/scripts/artifacts/validate-guest-image.sh
  '';

  meta = with lib; {
    description = "Agentic compute orchestration CLI";
    homepage = "https://github.com/spoke-sh/port";
    license = licenses.mit;
    maintainers = [ ];
    mainProgram = "port";
  };
}

{
  lib,
  firecracker,
  makeWrapper,
  stdenvNoCC,
}:
let
  hostKit = import ./nixos/aws-pvm-host-kit-contract.nix;
  hostKitManifest = builtins.toFile "aws-pvm-host-kit.json" (builtins.toJSON hostKit);
in
stdenvNoCC.mkDerivation {
  pname = hostKit.package.name;
  version = hostKit.package.version;

  dontUnpack = true;
  nativeBuildInputs = [ makeWrapper ];

  installPhase = ''
    mkdir -p $out/bin $out/share/port/nixos

    makeWrapper ${firecracker}/bin/firecracker $out/bin/${hostKit.firecracker_binary_name}
    ln -s ${firecracker}/bin/jailer $out/bin/jailer

    install -Dm644 ${hostKitManifest} \
      $out/share/port/aws-pvm-host-kit.json
    install -Dm644 ${./nixos/aws-pvm-host.nix} \
      $out/share/port/nixos/aws-pvm-host.nix
    install -Dm644 ${./nixos/aws-pvm-host-kit-contract.nix} \
      $out/share/port/nixos/aws-pvm-host-kit-contract.nix
  '';

  passthru = {
    inherit hostKit;
    modulePath = "/share/port/nixos/aws-pvm-host.nix";
    manifestPath = "/share/port/aws-pvm-host-kit.json";
  };

  meta = with lib; {
    description = "Port AWS x86_64 PVM host-kit package surface";
    homepage = "https://github.com/spoke-sh/port";
    license = licenses.mit;
    platforms = platforms.linux;
    mainProgram = hostKit.firecracker_binary_name;
  };
}

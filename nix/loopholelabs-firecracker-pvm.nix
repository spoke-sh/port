{
  lib,
  fetchurl,
  stdenvNoCC,
}:
let
  releaseTag = "release-main-live-migration-pvm";
  version = "1.13.0-dev+loopholelabs.pvm.7f6c070fa09c";
  firecracker = fetchurl {
    url = "https://github.com/loopholelabs/firecracker/releases/download/${releaseTag}/firecracker.linux-x86_64";
    hash = "sha256-CpWRh1J115AnT1yFRiQjTtsiODCZSYmOZ9g+WQpMzM0=";
  };
  jailer = fetchurl {
    url = "https://github.com/loopholelabs/firecracker/releases/download/${releaseTag}/jailer.linux-x86_64";
    hash = "sha256-CEVPR8pPlAoQVCe3D2iKMn/95TywLgL7WBPXzbmvIfo=";
  };
in
stdenvNoCC.mkDerivation {
  pname = "loopholelabs-firecracker-pvm";
  inherit version;

  dontUnpack = true;

  installPhase = ''
    mkdir -p $out/bin
    install -Dm755 ${firecracker} $out/bin/firecracker
    install -Dm755 ${jailer} $out/bin/jailer
  '';

  passthru = {
    inherit releaseTag;
    sourceRepo = "loopholelabs/firecracker";
  };

  meta = with lib; {
    description = "Pinned loopholelabs Firecracker PVM binaries for AWS no-KVM hosts";
    homepage = "https://github.com/loopholelabs/firecracker/releases/tag/release-main-live-migration-pvm";
    license = licenses.asl20;
    platforms = [ "x86_64-linux" ];
    mainProgram = "firecracker";
    sourceProvenance = with sourceTypes; [ binaryNativeCode ];
  };
}

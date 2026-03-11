set shell := ["bash", "-euo", "pipefail", "-c"]

default:
  @just --list --unsorted

# Cargo workflows
fmt:
  cargo fmt --all

fmt-check:
  cargo fmt --all -- --check

doctest *args:
  cargo test --doc {{args}}

clippy *args:
  cargo clippy --workspace --all-targets -- {{args}}

test *args:
  cargo test {{args}}

nextest *args:
  cargo nextest run {{args}}

coverage args="":
  mkdir -p coverage
  if [[ -n "{{args}}" ]]; then cargo llvm-cov {{args}}; else cargo llvm-cov --workspace --lcov --output-path coverage/lcov.info; fi

quality: fmt-check test
  @echo "quality checks passed"

# Keel workflows
keel *args:
  keel {{args}}

doctor:
  keel doctor

flow:
  keel flow

generate:
  keel generate

next:
  keel next --role operator

verify story:
  keel verify run {{story}}

# Port CLI workflows
port *args:
  cargo run -p port-cli -- {{args}}

demo-doctor config="examples/port.toml":
  cargo run -p port-cli -- --config {{config}} doctor

demo-build-kernel config="examples/port.toml" architecture="native" substrate="firecracker" protection="standard":
  cargo run -p port-cli -- --config {{config}} artifacts build --artifact demo-kernel --architecture {{architecture}} --substrate {{substrate}} --protection-mode {{protection}}

demo-validate-kernel config="examples/port.toml" architecture="native" substrate="firecracker" protection="standard":
  cargo run -p port-cli -- --config {{config}} artifacts validate --artifact demo-kernel --architecture {{architecture}} --substrate {{substrate}} --protection-mode {{protection}}

demo-build-guest config="examples/port.toml" architecture="native" substrate="firecracker" protection="standard":
  cargo run -p port-cli -- --config {{config}} artifacts build --artifact demo-guest --architecture {{architecture}} --substrate {{substrate}} --protection-mode {{protection}}

demo-validate-guest config="examples/port.toml" architecture="native" substrate="firecracker" protection="standard":
  cargo run -p port-cli -- --config {{config}} artifacts validate --artifact demo-guest --architecture {{architecture}} --substrate {{substrate}} --protection-mode {{protection}}

demo-push-oci config=".port/demo-oci.toml" registry="127.0.0.1:5510" registry_port="5510" container="port-demo-oci-registry":
  #!/usr/bin/env bash
  set -euo pipefail
  mkdir -p "$(dirname "{{config}}")"
  cp examples/port.toml "{{config}}"
  perl -0pi -e 's#\[artifacts\.kernels\.demo-kernel\.reference\]\nregistry = "demo-fs"#\[artifacts.kernels.demo-kernel.reference\]\nregistry = "{{registry}}"#' "{{config}}"
  perl -0pi -e 's#\[artifacts\.kernels\.demo-kernel\.distribution\.push\]\nbackend = "file-system"\nroot = "artifact-store/demo-fs"#\[artifacts.kernels.demo-kernel.distribution.push\]\nbackend = "oci-registry"\ntransport = "plain-http"\n\n[artifacts.kernels.demo-kernel.distribution.push.auth]\nkind = "anonymous"#' "{{config}}"
  perl -0pi -e 's#\[artifacts\.kernels\.demo-kernel\.distribution\.pull\]\nbackend = "file-system"\nroot = "artifact-store/demo-fs"#\[artifacts.kernels.demo-kernel.distribution.pull\]\nbackend = "oci-registry"\ntransport = "plain-http"\n\n[artifacts.kernels.demo-kernel.distribution.pull.auth]\nkind = "anonymous"#' "{{config}}"
  runtime="${PORT_OCI_DEMO_CONTAINER_RUNTIME:-docker}"
  if ! command -v "$runtime" >/dev/null 2>&1; then
    echo "demo-push-oci requires a container runtime; install docker or set PORT_OCI_DEMO_CONTAINER_RUNTIME" >&2
    exit 1
  fi
  if "$runtime" container inspect "{{container}}" >/dev/null 2>&1; then
    "$runtime" rm -f "{{container}}" >/dev/null
  fi
  "$runtime" run -d --rm --name "{{container}}" -p "{{registry_port}}:5000" registry:2 >/dev/null
  ready=0
  for _attempt in {1..20}; do
    if curl -fsS "http://{{registry}}/v2/" >/dev/null 2>&1; then
      ready=1
      break
    fi
    sleep 1
  done
  if [[ "$ready" -ne 1 ]]; then
    echo "local OCI registry '{{registry}}' did not become ready" >&2
    exit 1
  fi
  port_cli() {
    if command -v cargo >/dev/null 2>&1 && command -v oras >/dev/null 2>&1; then
      cargo run -p port-cli -- "$@"
    elif command -v nix >/dev/null 2>&1; then
      nix develop --command cargo run -p port-cli -- "$@"
    else
      echo "demo-push-oci requires cargo and oras on PATH, or nix to supply them" >&2
      exit 1
    fi
  }
  port_cli --config "{{config}}" artifacts build --artifact demo-kernel --architecture native
  port_cli --config "{{config}}" artifacts push --artifact demo-kernel --architecture native

demo-pull-oci config=".port/demo-oci.toml" container="port-demo-oci-registry":
  #!/usr/bin/env bash
  set -euo pipefail
  runtime="${PORT_OCI_DEMO_CONTAINER_RUNTIME:-docker}"
  if [[ ! -f "{{config}}" ]]; then
    echo "missing OCI demo config '{{config}}'; run 'just demo-push-oci' first" >&2
    exit 1
  fi
  if ! command -v "$runtime" >/dev/null 2>&1; then
    echo "demo-pull-oci requires a container runtime; install docker or set PORT_OCI_DEMO_CONTAINER_RUNTIME" >&2
    exit 1
  fi
  if ! "$runtime" container inspect "{{container}}" >/dev/null 2>&1; then
    echo "OCI demo registry '{{container}}' is not running; run 'just demo-push-oci' first" >&2
    exit 1
  fi
  arch="$(uname -m)"
  case "$arch" in
    x86_64|amd64) native_arch="x86_64" ;;
    aarch64|arm64) native_arch="aarch64" ;;
    *)
      echo "unsupported native architecture '$arch' for demo-pull-oci" >&2
      exit 1
      ;;
  esac
  artifact_path="artifacts/kernel/demo/${native_arch}/firecracker/standard/vmlinux"
  port_cli() {
    if command -v cargo >/dev/null 2>&1 && command -v oras >/dev/null 2>&1; then
      cargo run -p port-cli -- "$@"
    elif command -v nix >/dev/null 2>&1; then
      nix develop --command cargo run -p port-cli -- "$@"
    else
      echo "demo-pull-oci requires cargo and oras on PATH, or nix to supply them" >&2
      exit 1
    fi
  }
  test -s "$artifact_path"
  rm -f "$artifact_path"
  test ! -e "$artifact_path"
  port_cli --config "{{config}}" artifacts pull --artifact demo-kernel --architecture native
  test -s "$artifact_path"
  "$runtime" rm -f "{{container}}" >/dev/null

demo-launch machine="demo" config="examples/port.toml":
  cargo run -p port-cli -- --config {{config}} machine launch --machine {{machine}}

demo-status machine="demo" config="examples/port.toml":
  cargo run -p port-cli -- --config {{config}} machine status --machine {{machine}}

demo-stop machine="demo" config="examples/port.toml":
  cargo run -p port-cli -- --config {{config}} machine stop --machine {{machine}}

set shell := ["bash", "-euo", "pipefail", "-c"]

default:
  @just --list --unsorted

# Cargo workflows
fmt:
  cargo fmt --all

fmt-check:
  cargo fmt --all -- --check

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
doctor:
  keel doctor

flow:
  keel flow

generate:
  keel generate

next:
  keel next --agent

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

demo-launch machine="demo" config="examples/port.toml":
  cargo run -p port-cli -- --config {{config}} machine launch --machine {{machine}}

demo-status machine="demo" config="examples/port.toml":
  cargo run -p port-cli -- --config {{config}} machine status --machine {{machine}}

demo-stop machine="demo" config="examples/port.toml":
  cargo run -p port-cli -- --config {{config}} machine stop --machine {{machine}}

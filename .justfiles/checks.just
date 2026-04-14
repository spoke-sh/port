fmt:
  cargo fmt --all

fmt-check:
  cargo fmt --all -- --check

doctest *args:
  cargo test --doc {{args}}

clippy *args:
  cargo clippy --workspace --all-targets -- {{args}}

test *args:
  RUST_TEST_THREADS=1 cargo test {{args}}

nextest *args:
  cargo nextest run {{args}}

coverage args="":
  mkdir -p coverage
  if [[ -n "{{args}}" ]]; then cargo llvm-cov {{args}}; else cargo llvm-cov --workspace --lcov --output-path coverage/lcov.info; fi

check: fmt-check test doctest
  @echo "quality checks passed"

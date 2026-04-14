fmt:
  cargo fmt --all

fmt-check:
  cargo fmt --all -- --check

doctest *args:
  cargo test --doc {{args}}

clippy *args:
  cargo clippy --workspace --all-targets -- -D warnings {{args}}

test *args:
  cargo nextest run -j 1 {{args}}

coverage args="":
  mkdir -p coverage
  if [[ -n "{{args}}" ]]; then cargo llvm-cov {{args}}; else cargo llvm-cov --workspace --lcov --output-path coverage/lcov.info; fi

check: fmt-check test doctest
  @echo "quality checks passed"

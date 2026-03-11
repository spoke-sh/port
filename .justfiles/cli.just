build:
  cd {{justfile_directory()}} && cargo build -p port-cli

run *args:
  cd {{justfile_directory()}} && cargo run -p port-cli -- {{args}}

build:
  if command -v nix >/dev/null 2>&1 && [ -f {{justfile_directory()}}/flake.nix ]; then cd {{justfile_directory()}} && nix develop -c cargo build -p port-cli; else cd {{justfile_directory()}} && cargo build -p port-cli; fi

[positional-arguments]
run *args:
  if command -v nix >/dev/null 2>&1 && [ -f {{justfile_directory()}}/flake.nix ]; then cd {{justfile_directory()}} && nix develop -c cargo run -p port-cli -- "$@"; else cd {{justfile_directory()}} && cargo run -p port-cli -- "$@"; fi

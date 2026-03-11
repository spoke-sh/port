doctor:
  if command -v nix >/dev/null 2>&1; then nix develop {{justfile_directory()}} -c keel doctor; else keel doctor; fi

flow:
  if command -v nix >/dev/null 2>&1; then nix develop {{justfile_directory()}} -c keel flow; else keel flow; fi

generate:
  if command -v nix >/dev/null 2>&1; then nix develop {{justfile_directory()}} -c keel generate; else keel generate; fi

next:
  if command -v nix >/dev/null 2>&1; then nix develop {{justfile_directory()}} -c keel next --role operator; else keel next --role operator; fi

verify story:
  if command -v nix >/dev/null 2>&1; then nix develop {{justfile_directory()}} -c keel verify run {{story}}; else keel verify run {{story}}; fi

set shell := ["bash", "-euo", "pipefail", "-c"]

mod board '.justfiles/board.just'
mod checks '.justfiles/checks.just'
mod signal '.justfiles/signal.just'
mod cli '.justfiles/cli.just'
mod demo '.justfiles/demo.just'

default:
  @printf '%s\n' \
    'Common recipes:' \
    '  just mission               Show mission artifact report' \
    '  just docs-install          Install docs site dependencies' \
    '  just docs-dev              Run the public docs site locally' \
    '  just docs-build            Build the public docs site' \
    '  just check                 Run formatting, tests, and doctests' \
    '  keel doctor                Validate the Keel board' \
    '  keel flow --scene          Show workflow lane state' \
    '  keel mission next --status Show next board moves' \
    '  just test                  Run workspace tests' \
    '  just build                 Build the port CLI binary' \
    '  just package <target>      Build a canonical install tarball' \
    '  just package-proof <tgt>   Prove packaged Port works from an installed path' \
    '  just port --help           Run the Port CLI in the dev shell' \
    '' \
    'More recipes:' \
    '  just --list board' \
    '  just --list checks' \
    '  just --list signal' \
    '  just --list cli' \
    '  just --list demo'

mission *args:
  @bash {{justfile_directory()}}/scripts/keel-mission-show.sh {{args}}

docs-install:
  nix shell nixpkgs#nodejs_22 -c sh -lc 'cd website && npm install'

docs-dev:
  nix shell nixpkgs#nodejs_22 -c sh -lc 'cd website && npm run start -- --host "${HOST:-0.0.0.0}" --port "${PORT:-3000}"'

docs-build:
  nix shell nixpkgs#nodejs_22 -c sh -lc 'cd website && npm run build'

screen *args:
  if command -v nix >/dev/null 2>&1; then nix develop {{justfile_directory()}} -c keel screen {{args}}; else keel screen {{args}}; fi

check:
  @just checks::check

quality:
  @just checks::check

test *args:
  @just checks::test {{args}}

doctest *args:
  @just checks::doctest {{args}}

doctor:
  @just board::doctor

flow:
  @just board::flow

build:
  @just cli::build

[positional-arguments]
package *args:
  @just cli::package "$@"

[positional-arguments]
package-proof *args:
  @just cli::package-proof "$@"

[positional-arguments]
port *args:
  @just cli::run "$@"

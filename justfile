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
    '  just check                 Run formatting, tests, and doctests' \
    '  just doctor                Validate the Keel board' \
    '  just flow                  Show workflow lane state' \
    '  just test                  Run workspace tests' \
    '  just build                 Build the port CLI binary' \
    '  just package <target>      Build a canonical install tarball' \
    '  just package-proof <tgt>   Prove packaged Port works from an installed path' \
    '  just port --help           Run the Port CLI in the dev shell' \
    '  just keel flow             Run arbitrary Keel commands' \
    '' \
    'More recipes:' \
    '  just --list board' \
    '  just --list checks' \
    '  just --list signal' \
    '  just --list cli' \
    '  just --list demo'

keel *args:
  if command -v nix >/dev/null 2>&1; then nix develop {{justfile_directory()}} -c keel {{args}}; else keel {{args}}; fi

mission *args:
  @bash {{justfile_directory()}}/scripts/keel-mission-show.sh {{args}}

screen *args:
  @just mission {{args}}

check:
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

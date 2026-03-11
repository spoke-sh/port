set shell := ["bash", "-euo", "pipefail", "-c"]

mod board '.justfiles/board.just'
mod checks '.justfiles/checks.just'
mod signal '.justfiles/signal.just'
mod cli '.justfiles/cli.just'
mod demo '.justfiles/demo.just'

default:
  @printf '%s\n' \
    'Common recipes:' \
    '  just mission               Run repo verification and show mission signal' \
    '  just quality               Run formatting, tests, and doctests' \
    '  just doctor                Validate the Keel board' \
    '  just flow                  Show workflow lane state' \
    '  just test                  Run workspace tests' \
    '  just build                 Build the port CLI binary' \
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
  @just quality
  @just doctor
  @just signal::report {{args}}

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

port *args:
  @just cli::run {{args}}

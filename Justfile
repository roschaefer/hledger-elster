set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list

help:
    @just --list

build:
    cargo build --release

run *args="":
    @args="{{args}}"; args="${args#-- }"; cargo run -- $args

test:
    cargo test

test-scrut *paths="tests/specifications":
    cargo build
    PATH="$(realpath "${CARGO_TARGET_DIR:-target}/debug"):$PATH" scrut test --combine-output {{paths}}

# Print a specification's commands as a bash script, without Scrut's prompts
scenario-script document:
    @printf '#!/usr/bin/env bash\nTESTDIR=%q\n' "$(realpath "$(dirname "{{document}}")")"
    @awk '/^```/ { inside = !inside && /^```scrut([[:space:]{]|$)/; next } \
      inside && /^\$ / { print substr($0, 3); next } \
      inside && /^>( |$)/ { print substr($0, 3) }' "{{document}}"

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy --all-targets -- -D warnings

machete:
    cargo machete

audit:
    cargo audit

check: fmt-check clippy machete audit test

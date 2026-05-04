set shell := ["bash", "-cu"]

default:
    @just --list

help:
    @just --list

sync:
    uv sync --extra test

test:
    uv run pytest tests/ -v

generate-features:
    uv run python scripts/generate_features.py

acceptance: generate-features
    uv run behave tests/features

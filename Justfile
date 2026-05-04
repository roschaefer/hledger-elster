set shell := ["bash", "-cu"]
export UV_CACHE_DIR := ".uv-cache"

default:
    @just --list

help:
    @just --list

sync:
    uv sync --extra test

test:
    uv run pytest tests/ -v

lint:
    uv run ruff check .

format:
    uv run ruff format .

check-format:
    uv run ruff format --check .

typecheck:
    uv run ty check --exclude tests/features/steps

generate-features:
    uv run python scripts/generate_features.py

acceptance: generate-features
    uv run behave tests/features

check: check-format lint typecheck test acceptance

set shell := ["bash", "-cu"]

default:
    @just --list

help:
    @just --list

test:
    pytest tests/ -v

generate-features:
    python scripts/generate_features.py

acceptance: generate-features
    behave tests/features

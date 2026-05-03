set shell := ["bash", "-cu"]

default:
    @just --list

help:
    @just --list

test:
    pytest tests/ -v

#!/usr/bin/env bash
set -e

# assume we're in venv
maturin develop -m rust_chess/Cargo.toml && python3 test.py
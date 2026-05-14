#!/usr/bin/env bash
set -e

# assume we're in venv
.venv/bin/maturin develop -m rust_chess/Cargo.toml && .venv/bin/python test.py && RUST_CHESS="1" .venv/bin/python test.py
#!/usr/bin/env bash
set -e

# assume we're in venv
maturin develop -m rust_chess/Cargo.toml && python test.py && RUST_CHESS="1" python test.py
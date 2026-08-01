#!/usr/bin/env bash
set -e

# Activate virtualenv if not already active
if [ -z "$VIRTUAL_ENV" ]; then
  if [ -d ".venv" ]; then
    source .venv/bin/activate
  elif [ -d "env" ]; then
    source env/bin/activate
  elif [ -d "venv" ]; then
    source venv/bin/activate
  fi
fi

maturin develop --manifest-path=pyrust_chess/Cargo.toml --group=pyrust_chess/pyproject.toml:dev && python test.py && pyrust_chess="1" python test.py
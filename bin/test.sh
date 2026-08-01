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

(cd pyrust_chess && maturin develop) && python test.py && pyrust_chess="1" python test.py
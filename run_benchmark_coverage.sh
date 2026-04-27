#!/usr/bin/env bash
set -e

# Use the existing myenv or create it
if [ ! -d "myenv" ]; then
    python3.14 -m venv myenv
fi
source myenv/bin/activate

# Install requirements
pip install -e . asv coverage maturin &&  maturin develop -m rust_chess/Cargo.toml

# Configure coverage
cat << 'EOF' > .coveragerc
[run]
source = chess
parallel = True
concurrency = multiprocessing
sigterm = True

[report]
exclude_lines =
    pragma: no cover
    def __repr__
EOF

# Clean previous coverage data
coverage erase || true
rm -f .coverage.*
rm -f .coverage

echo "Running benchmarks directly with python script for accurate coverage..."
coverage run run_benchmark_coverage.py

# Combine parallel reports and display
echo "Combining and generating coverage reports..."
coverage combine || true
coverage report -m
coverage html

echo "Coverage HTML report available in htmlcov/index.html"
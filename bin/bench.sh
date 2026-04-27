

# TODO, remove the --quick to get thorough bench
asv run --quick --show-stderr --machine Kraktus --bench PieceSuite --bench MoveSuite --bench SquareSetSuite --bench BaseBoardSuite && \
asv publish && asv preview && open localhost:8080
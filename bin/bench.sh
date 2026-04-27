

asv run --show-stderr --machine Kraktus --bench PieceSuite --bench MoveSuite --bench SquareSetSuite && \
asv publish && asv preview && open localhost:8080
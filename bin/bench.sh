

asv run --show-stderr --machine Kraktus --bench PieceSuite --bench MoveSuite --bench SquareSetSuite --bench BaseBoardSuite && \
asv publish && asv preview && open localhost:8080
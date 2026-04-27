

- When porting API to rust, always use the highest-level of abstraction of shakmaty internal, do not fallback to constants.
- Also use shakmaty constant every time it is possible, like for default board fen, full bitboard, etc.
- Never implement private python API (starting with an underscore)
- Use types in rust_chess/src/utils.py for converting args of python method to higher-level types:


Exemple:
    * use PyRole instead of u8 for piece_type
    * use PyColor instead of bool for color
    * use PySquare instead of u32/any int for square/sq

- Never modify chess/__init__.py
- Never modify test.py
- All your temporary scripts/tests should be in a tmp folder- When deriving `Default` for board-like structures, ensure it returns the standard chess starting position (e.g. `shakmaty::Board::default()`), not an empty board, as PyO3 will use this for the Python `__new__` default without arguments.

## Quick Start / Context for New Sessions

**Project Structure:**
- `rust_chess/`: The Rust extension crate. Contains `Cargo.toml` and the `src/` directory where the PyO3 bindings and Rust implementations reside.
- `chess/`: The original `python-chess` Python library. We avoid modifying this directly, aiming to monkey-patch it via `rust_chess`.
- `test.py`: The main `python-chess` test suite.
- `bin/test.sh`: The standard script to build the Rust extension and run the test suite (both purely in Python and with the Rust extension enabled).
- `BOARD_PLAN.md`: The current architectural plan for porting `chess.Board`.

**Environment & Commands:**
- Always run tests and builds within the virtual environment (`.venv`).
- **Build the extension**: `maturin develop -m rust_chess/Cargo.toml` (or `pip install -e rust_chess` if maturin behaves inconsistently, though the `test.sh` script handles this).
- **Run the full test suite**: `bash bin/test.sh` (this will build the extension, run `python3 test.py` normally, and then run `RUST_CHESS=1 python3 test.py`).
- **Run quick checks**: Use `.venv/bin/python test.py` directly, or specific test cases like `.venv/bin/python test.py SquareSetTestCase`. Remember to set `RUST_CHESS=1` to test the Rust integration.

**Architectural Paradigm:**
- `python-chess` allows users to create transient invalid board states (e.g., removing a king temporarily, or manually setting bitboards to overlapping states). 
- `shakmaty` (our Rust backend) is highly optimized but strictly rejects invalid states.
- **Solution**: We store raw bitboards (`by_role`, `by_color`) in the Rust structs (like `BaseBoard`) and only instantiate strict `shakmaty` types (like `shakmaty::Board` or `shakmaty::Chess`) on the fly when absolutely necessary for complex validation or move generation.


## Guidelines


- When porting API to rust, always use the highest-level of abstraction of shakmaty internal, do not fallback to constants. You can check the whole shakmaty API by viewing `shakmaty_api.txt`
- USE Bitboards as much as possible. NEVER ENCODE BITBOARD Constants as raw u64, always look for an appropriely named constant in shakmaty::Bitboard.
- Also use shakmaty constant every time it is possible, like for default board fen, full bitboard, etc.
- Never implement private python API (starting with an underscore)

- Use types in rust_chess/src/utils.py for converting args of python method to higher-level types:


Exemple:
    * use PyRole instead of u8 for piece_type
    * use PyColor instead of bool for color
    * use PySquare instead of u32/any int for square/sq

- DO NOT MONKEY-PATCH Board and BaseBoard. They will be swapped later on by user at the import level, like :

```py
USE_RUST_CHESS = os.environ.get("RUST_CHESS") == "1"
if USE_RUST_CHESS:
    import rust_chess
    rust_chess.patch_supported(
        dst_module=chess,
        src_module=rust_chess,
    )
    from rust_chess import BaseBoard
    from chess import Board
else:
    from chess import BaseBoard, Board
```

- Never modify chess/\*.py
- Never modify test.py
- All your temporary scripts/tests should be in a tmp/ folder.

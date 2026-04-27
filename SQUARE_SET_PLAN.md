# Plan to Port SquareSet to Rust (rust_chess)

## Goal
Port all methods of the `SquareSet` class from `chess/__init__.py` to a Rust extension module `rust_chess`, using `shakmaty::Bitboard` internally for optimal performance. The external Python API, behavior, and duck-typing must remain 100% compatible with the current `python-chess` implementation. For now keep the `chess/__init__.py` unmodified, everything will be ported at once at the end of the port process.

## 1. Class Definition & Initialization
- Define `SquareSet` in `rust_chess/src/lib.rs` with `#[pyclass]`.
- Store the internal state as `pub bb: shakmaty::Bitboard`.
- Implement `__new__` (or `__init__`) accepting an optional `squares` argument which can be an integer mask, another `SquareSet`, or an iterable of squares.
- Implement the `mask` property manually with a `#[getter]` and `#[setter]` that extracts the `u64` from `bb.0` for backwards compatibility (since `Bitboard` itself is not a `pyclass`).

# STRATEGY to implement each method

- Build upon the current 
- NEVER REIMPLEMENT THE FUNCTION logic if you can reuse a method of Bitboard.
- check the online documentation: https://docs.rs/shakmaty/latest/shakmaty/bitboard/struct.Bitboard.html

## 2. Basic Container and Set Protocol (`Set`, `MutableSet`, `frozenset`)
- `__contains__`: Check if a square is in the bitboard. Ensure square bounds checking (0 <= square < 64) to avoid panics.
- `__len__`: Return the popcount using `self.bb.count()`.
- `__iter__`: Implement an iterator `SquareSetIter` (scanning forward) wrapping `Bitboard`'s iterator.
- `__reversed__`: Implement a reverse iterator `SquareSetRevIter` (scanning backward manually via `.leading_zeros()`).
- `add`, `discard`, `remove`, `pop`, `clear`: Mutate the internal bitboard.
  - **CRITICAL**: `pop` must be deterministic: it must remove and return the Least Significant Bit (LSB) to match Python's behavior exactly, using `Bitboard::first()` or `.trailing_zeros()`.
  - Raise `KeyError` in `remove` and `pop` appropriately.
  - Ensure bounds checking before converting to `shakmaty::Square`.

## 3. Set Operations (Logic & Operators)
- Implement `isdisjoint`, `issubset`, `issuperset`.
- Implement `union` / `__or__`, `intersection` / `__and__`, `difference` / `__sub__`, `symmetric_difference` / `__xor__`.
- Implement `update`, `intersection_update`, `difference_update`, `symmetric_difference_update` using `#[pyo3(signature = (*others))]` to handle variadic arguments properly.
- Implement `__ior__`, `__iand__`, `__isub__`, `__ixor__` (these take exactly one argument).
- Extraction Helper: Support accepting either another `SquareSet`, an object with `__int__()`, or an iterable. The extraction must try `__int__()` first, mask it with `0xffff_ffff_ffff_ffff`, and if that fails, try iterating to build the mask.

## 4. Bitwise Shift and Inversion
- `__lshift__`, `__rshift__`, `__ilshift__`, `__irshift__`: Shift the bitboard. **CRITICAL:** Explicitly check if `shift >= 64` and return `0` to prevent Rust thread panics, matching Python's safe overflow behavior.
- `__invert__`: Bitwise NOT, masked with `BB_ALL` (`0xffff_ffff_ffff_ffff`).

## 5. Type Conversions & Formatting
- `__bool__`: Return true if mask != 0.
- `__int__`, `__index__`: Return the `u64` mask as a Python integer.
- `__eq__`: Compare masks. Return `NotImplemented` for incompatible types.
- `__repr__`: Format as `SquareSet(0x...)`.
- `__str__`: Return the multi-line string representation (dot and 1 matrix).
- `tolist`: Convert to a list of 64 booleans.
- `copy`: Return a new `SquareSet` with the same mask.
- `_repr_svg_`: Do not reimplement it

## 6. Chess-Specific Features & Classmethods
- `carry_rippler`: Return an iterator over subsets using `shakmaty::Bitboard::carry_rippler`.
- `mirror`: Return a vertically flipped bitboard using `flip_vertical()`.
- `@classmethod ray(a, b)`: Use `shakmaty::attacks::ray`. Must validate `0 <= a, b < 64` before calling `Square::new()` to prevent panics.
- `@classmethod between(a, b)`: Use `shakmaty::attacks::between`. Must validate bounds.
- `@classmethod from_square(square)`: Create a `SquareSet` with a single bit set. Must validate bounds.

## Clarifying Questions (Resolved)
1. No fallback to pure Python for `SquareSet`.
2. `_repr_svg_` will not be implemented for now.
3. Enforce aggressive duck-typing using `__int__()` and iterators, just like python-chess does.

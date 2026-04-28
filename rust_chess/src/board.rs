use shakmaty::{Bitboard, Color, FromSetup, Position, Square};

use std::num::NonZeroU32;

use crate::base_board::BaseBoard;
use crate::py_move::PyMove;
use crate::util::{PyColor, PySquare};
use pyo3::ffi::PyObject;
use pyo3::prelude::*;

#[pyclass(module = "rust_chess", name = "LegalMoveGenerator")]
pub struct LegalMoveGenerator {
    board: Py<Board>,
}

#[pymethods]
impl LegalMoveGenerator {
    #[new]
    fn py_new(board: Py<Board>) -> Self {
        Self { board }
    }

    fn __bool__(&self, py: Python<'_>) -> PyResult<bool> {
        let board = self.board.bind(py);
        let chess = Board::try_shakmaty(&board)?;
        Ok(!chess.legal_moves().is_empty())
    }

    fn __len__(&self, py: Python<'_>) -> PyResult<usize> {
        self.count(py)
    }

    fn count(&self, py: Python<'_>) -> PyResult<usize> {
        let board = self.board.bind(py);
        let chess = Board::try_shakmaty(&board)?;
        Ok(chess.legal_moves().len())
    }

    fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::PyAny>> {
        let board = self.board.bind(py);
        let chess = Board::try_shakmaty(&board)?;

        let mode = shakmaty::CastlingMode::Standard; // TODO: support chess960
        let mut py_moves = Vec::new();
        for m in chess.legal_moves() {
            let pm = PyMove {
                inner: m.to_uci(mode),
            };
            py_moves.push(Bound::new(py, pm)?.into_any());
        }

        let list = pyo3::types::PyList::new(py, py_moves)?;
        list.call_method0("__iter__")
    }

    fn __contains__(&self, move_obj: &Bound<'_, pyo3::PyAny>, py: Python<'_>) -> PyResult<bool> {
        if let Ok(m) = move_obj.extract::<PyRef<'_, PyMove>>() {
            let board = self.board.bind(py);
            let chess = Board::try_shakmaty(&board)?;
            if let Ok(sm_move) = m.inner.to_move(&chess) {
                return Ok(chess.is_legal(sm_move));
            }
        }
        Ok(false)
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        let py = slf.py();
        let self_rust = slf.borrow();
        let board = self_rust.board.bind(py);
        let chess = Board::try_shakmaty(&board)?;
        let moves = chess.legal_moves();
        let mut sans = Vec::new();
        for m in moves.iter() {
            sans.push(shakmaty::san::San::from_move(&chess, m.clone()).to_string());
        }
        Ok(format!(
            "<LegalMoveGenerator at {:#x} ({})>",
            slf.as_ptr() as usize,
            sans.join(", ")
        ))
    }
}

#[pyclass(extends=BaseBoard, subclass, dict)]
pub struct Board {
    pub turn: Color,
    pub castling_rights: Bitboard,
    pub ep_square: Option<Square>,
    pub halfmove_clock: u16,
    pub fullmove_number: NonZeroU32,
    pub move_stack: Py<pyo3::types::PyList>,
    pub _stack: Py<pyo3::types::PyList>,
    pub chess960: bool,
}

#[pymethods]
impl Board {
    #[getter]
    fn chess960(&self) -> bool {
        self.chess960
    }

    #[setter]
    fn set_chess960(&mut self, chess960: bool) {
        self.chess960 = chess960;
    }

    #[getter]
    fn move_stack(&self, py: Python<'_>) -> Py<pyo3::types::PyList> {
        self.move_stack.clone_ref(py)
    }

    #[setter]
    fn set_move_stack(&mut self, stack: Py<pyo3::types::PyList>) {
        self.move_stack = stack;
    }

    #[getter]
    fn _stack(&self, py: Python<'_>) -> Py<pyo3::types::PyList> {
        self._stack.clone_ref(py)
    }

    #[setter]
    fn set__stack(&mut self, stack: Py<pyo3::types::PyList>) {
        self._stack = stack;
    }
    #[new]
    #[pyo3(signature = (fen=Some("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"), *, chess960=false))]
    #[allow(unused_variables)]
    fn __new__(py: Python<'_>, fen: Option<&str>, chess960: bool) -> PyResult<(Self, BaseBoard)> {
        let mut turn = shakmaty::Color::White;
        let mut castling_rights = shakmaty::Bitboard::EMPTY;
        let mut ep_square = None;
        let mut halfmove_clock = 0;
        let mut fullmove_number = 1;

        let base_board = if let Some(f) = fen {
            let setup = shakmaty::fen::Fen::from_ascii(f.as_bytes())
                .map_err(|e| {
                    pyo3::exceptions::PyValueError::new_err(format!("invalid fen: {}", e))
                })?
                .into_setup();

            turn = setup.turn;
            castling_rights = setup.castling_rights;
            ep_square = setup.ep_square;
            halfmove_clock = setup.halfmoves as u16;
            fullmove_number = setup.fullmoves.into();

            let (roles, colors) = setup.board.into_bitboards();
            BaseBoard {
                by_role: roles,
                by_color: colors,
                promoted: setup.promoted,
            }
        } else {
            let b = BaseBoard::empty()?;
            b
        };

        let board = Self {
            turn,
            castling_rights,
            ep_square,
            halfmove_clock,
            fullmove_number: std::num::NonZeroU32::new(fullmove_number)
                .unwrap_or(std::num::NonZeroU32::new(1).unwrap()),
            move_stack: pyo3::types::PyList::empty(py).into(),
            _stack: pyo3::types::PyList::empty(py).into(),
            chess960,
        };

        Ok((board, base_board))
    }

    #[pyo3(signature = (fen=Some("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"), *, chess960=false))]
    #[allow(unused_variables)]
    fn __init__(mut slf: PyRefMut<'_, Self>, fen: Option<&str>, chess960: bool) -> PyResult<()> {
        if let Some(f) = fen {
            println!("RUST INIT FEN: {:?}", f);
            let setup = shakmaty::fen::Fen::from_ascii(f.as_bytes())
                .map_err(|e| {
                    pyo3::exceptions::PyValueError::new_err(format!("invalid fen: {}", e))
                })?
                .into_setup();

            slf.turn = setup.turn;
            slf.castling_rights = setup.castling_rights;
            slf.ep_square = setup.ep_square;
            slf.halfmove_clock = setup.halfmoves as u16;
            slf.fullmove_number = std::num::NonZeroU32::new(setup.fullmoves.into())
                .unwrap_or(std::num::NonZeroU32::new(1).unwrap());

            let (roles, colors) = setup.board.into_bitboards();
            let promoted = setup.promoted;
            slf.move_stack.bind(slf.py()).call_method0("clear")?;
            slf._stack.bind(slf.py()).call_method0("clear")?;
            slf.chess960 = chess960;

            let mut base = slf.into_super();
            base.by_role = roles;
            base.by_color = colors;
            base.promoted = promoted;
        } else {
            slf.turn = shakmaty::Color::White;
            slf.castling_rights = shakmaty::Bitboard::EMPTY;
            slf.ep_square = None;
            slf.halfmove_clock = 0;
            slf.fullmove_number = std::num::NonZeroU32::new(1).unwrap();
            slf.move_stack.bind(slf.py()).call_method0("clear")?;
            slf._stack.bind(slf.py()).call_method0("clear")?;
            slf.chess960 = chess960;

            let mut base = slf.into_super();
            base.clear_board();
        }

        Ok(())
    }

    #[getter]
    fn turn(&self) -> bool {
        self.turn.is_white()
    }

    #[setter]
    fn set_turn(&mut self, turn: PyColor) {
        self.turn = turn.0
    }

    #[getter]
    fn castling_rights(&self) -> u64 {
        self.castling_rights.0
    }

    #[setter]
    fn set_castling_rights(&mut self, castling_rights: u64) {
        self.castling_rights = shakmaty::Bitboard(castling_rights);
    }

    #[getter]
    fn ep_square(&self) -> Option<u32> {
        self.ep_square.map(|sq| u32::from(sq))
    }

    #[setter]
    fn set_ep_square(&mut self, ep_square: Option<u32>) {
        self.ep_square = ep_square.map(|sq| shakmaty::Square::new(sq));
    }

    #[getter]
    fn halfmove_clock(&self) -> u16 {
        self.halfmove_clock
    }

    #[setter]
    fn set_halfmove_clock(&mut self, halfmove_clock: u16) {
        self.halfmove_clock = halfmove_clock;
    }

    #[getter]
    fn fullmove_number(&self) -> u16 {
        self.fullmove_number.get() as u16
    }

    #[setter]
    fn set_fullmove_number(&mut self, fullmove_number: u16) {
        self.fullmove_number = NonZeroU32::new(std::cmp::max(1, fullmove_number as u32)).unwrap();
    }

    fn clear(mut slf: PyRefMut<'_, Self>) -> PyResult<()> {
        slf.turn = shakmaty::Color::White;
        slf.castling_rights = shakmaty::Bitboard::EMPTY;
        slf.ep_square = None;
        slf.halfmove_clock = 0;
        slf.fullmove_number = NonZeroU32::new(1).unwrap();
        slf.move_stack.bind(slf.py()).call_method0("clear")?;
        slf._stack.bind(slf.py()).call_method0("clear")?;

        let mut base = slf.into_super();
        (&mut *base).clear_board();
        Ok(())
    }

    fn reset(mut slf: PyRefMut<'_, Self>) -> PyResult<()> {
        slf.turn = shakmaty::Color::White;
        slf.castling_rights = shakmaty::Bitboard(0x8100_0000_0000_0081); // standard castling rights
        slf.ep_square = None;
        slf.halfmove_clock = 0;
        slf.fullmove_number = NonZeroU32::new(1).unwrap();
        slf.move_stack.bind(slf.py()).call_method0("clear")?;
        slf._stack.bind(slf.py()).call_method0("clear")?;

        let mut base = slf.into_super();
        (&mut *base).reset_board();
        Ok(())
    }

    fn set_fen(mut slf: PyRefMut<'_, Self>, fen: &str) -> PyResult<()> {
        let setup = shakmaty::fen::Fen::from_ascii(fen.as_bytes())
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("invalid fen: {}", e)))?
            .into_setup();

        slf.turn = setup.turn;
        slf.castling_rights = setup.castling_rights;
        slf.ep_square = setup.ep_square;
        slf.halfmove_clock = setup.halfmoves as u16;
        slf.fullmove_number = setup.fullmoves.into();
        slf.move_stack.bind(slf.py()).call_method0("clear")?;
        slf._stack.bind(slf.py()).call_method0("clear")?;

        let mut base = slf.into_super();
        let (roles, colors) = setup.board.into_bitboards();
        base.by_role = roles;
        base.by_color = colors;
        base.promoted = setup.promoted;

        Ok(())
    }

    fn set_epd(slf: PyRefMut<'_, Self>, epd: &str) -> PyResult<()> {
        Board::set_fen(slf, epd)
    }

    #[pyo3(signature = (*, stack=None))]
    fn copy<'py>(
        slf: &Bound<'py, Self>,
        py: Python<'py>,
        stack: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, Self>> {
        let board_rust = slf.borrow();
        let base_rust = slf.as_super().borrow();

        let move_stack_bound = board_rust.move_stack.bind(py);
        let _stack_bound = board_rust._stack.bind(py);

        let mut stack_len = move_stack_bound.len();
        if let Some(s) = stack {
            if let Ok(b) = s.extract::<bool>() {
                if !b {
                    stack_len = 0;
                }
            } else if let Ok(i) = s.extract::<usize>() {
                stack_len = i;
            }
        }

        let new_move_stack = pyo3::types::PyList::empty(py);
        let stack_start = move_stack_bound.len().saturating_sub(stack_len);
        for i in stack_start..move_stack_bound.len() {
            let m = move_stack_bound.get_item(i)?;
            let copy_mod = py.import("copy")?;
            let copy_m = copy_mod.call_method1("copy", (m,))?;
            new_move_stack.append(copy_m)?;
        }

        let new__stack = pyo3::types::PyList::empty(py);
        let _stack_start = _stack_bound.len().saturating_sub(stack_len);
        for i in _stack_start.._stack_bound.len() {
            let s = _stack_bound.get_item(i)?;
            new__stack.append(s)?;
        }

        let new_board = Board {
            turn: board_rust.turn,
            castling_rights: board_rust.castling_rights,
            ep_square: board_rust.ep_square,
            halfmove_clock: board_rust.halfmove_clock,
            fullmove_number: board_rust.fullmove_number,
            move_stack: new_move_stack.into(),
            _stack: new__stack.into(),
            chess960: board_rust.chess960,
        };
        let new_base = base_rust.clone();

        Bound::new(py, (new_board, new_base))
    }

    #[pyo3(signature = (*, stack=None))]
    fn __copy__<'py>(
        slf: &Bound<'py, Self>,
        py: Python<'py>,
        stack: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, Self>> {
        Self::copy(slf, py, stack)
    }

    #[pyo3(signature = (memo, *, stack=None))]
    fn __deepcopy__<'py>(
        slf: &Bound<'py, Self>,
        py: Python<'py>,
        memo: Bound<'py, PyAny>,
        stack: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, Self>> {
        // deepcopy in python uses copy() essentially, we'll just do shallow for _stack.
        Self::copy(slf, py, stack)
    }

    #[getter]
    fn legal_moves<'py>(
        slf: &Bound<'py, Self>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, pyo3::PyAny>> {
        let generator = Bound::new(py, LegalMoveGenerator::py_new(slf.clone().unbind()))?;
        Ok(generator.into_any())
    }

    #[getter]
    fn pseudo_legal_moves<'py>(
        slf: &Bound<'py, Self>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, pyo3::PyAny>> {
        let chess_mod = py.import("chess")?;
        let generator = chess_mod.getattr("PseudoLegalMoveGenerator")?;
        generator.call1((slf,))
    }

    fn is_check(slf: &Bound<'_, Self>) -> PyResult<bool> {
        Ok(Board::try_shakmaty(slf)?.is_check())
    }

    fn is_variant_end(slf: &Bound<'_, Self>) -> PyResult<bool> {
        Ok(Board::try_shakmaty(slf)?.is_variant_end())
    }

    #[pyo3(signature = (square, piece, promoted=false))]
    fn set_piece_at(
        mut slf: PyRefMut<'_, Self>,
        py: Python<'_>,
        square: PySquare,
        piece: Option<crate::piece::PyPiece>,
        promoted: bool,
    ) {
        slf.clear_stack(py);
        slf.into_super().set_piece_at(square, piece, promoted);
    }

    fn remove_piece_at(
        mut slf: PyRefMut<'_, Self>,
        py: Python<'_>,
        square: PySquare,
    ) -> Option<crate::piece::PyPiece> {
        slf.clear_stack(py);
        slf.into_super().remove_piece_at(square)
    }

    fn set_board_fen(mut slf: PyRefMut<'_, Self>, py: Python<'_>, fen: &str) -> PyResult<()> {
        slf.clear_stack(py);
        slf.into_super().set_board_fen(fen)
    }

    fn set_chess960_pos(slf: &Bound<'_, Self>, py: Python<'_>, scharnagl: u16) -> PyResult<()> {
        slf.borrow_mut().clear_stack(py);
        slf.call_method1("_set_chess960_pos", (scharnagl,))?;

        let mut rust_board = slf.borrow_mut();
        rust_board.chess960 = true;
        rust_board.turn = shakmaty::Color::White;
        let rooks = rust_board.into_super().by_role.get(shakmaty::Role::Rook).0;

        let mut rust_board2 = slf.borrow_mut();
        rust_board2.castling_rights = shakmaty::Bitboard(rooks);
        rust_board2.ep_square = None;
        rust_board2.halfmove_clock = 0;
        rust_board2.fullmove_number = std::num::NonZeroU32::new(1).unwrap();
        Ok(())
    }

    fn apply_mirror(slf: &Bound<'_, Self>, py: Python<'_>) -> PyResult<()> {
        let turn = !slf.borrow().turn;
        slf.borrow_mut().turn = turn;
        let flip_vertical = py.import("chess")?.getattr("flip_vertical")?;
        slf.borrow_mut()
            .into_super()
            .apply_transform(&flip_vertical)?;
        Ok(())
    }

    fn clear_board(mut slf: PyRefMut<'_, Self>, py: Python<'_>) {
        slf.clear_stack(py);
        slf.into_super().clear_board();
    }

    fn reset_board(mut slf: PyRefMut<'_, Self>, py: Python<'_>) {
        slf.clear_stack(py);
        slf.into_super().reset_board();
    }

    fn clear_stack(&mut self, py: Python<'_>) {
        self.move_stack = pyo3::types::PyList::empty(py).into();
        self._stack = pyo3::types::PyList::empty(py).into();
    }

    fn mirror(slf: &Bound<'_, Self>) -> PyResult<Py<PyAny>> {
        let py = slf.py();
        let py_board = slf.call_method0("copy")?;
        py_board.call_method0("apply_mirror")?;
        Ok(py_board.into_any().unbind())
    }

    #[pyo3(signature = (*, claim_draw=None))]
    #[allow(unused_variables)]
    fn is_game_over(slf: &Bound<'_, Self>, claim_draw: Option<bool>) -> PyResult<bool> {
        Ok(Board::try_shakmaty(slf)?.is_game_over())
    }
}

impl Board {
    // &Bound<'_, Self> to be able to acess BaseBoard
    fn try_shakmaty(slf: &Bound<'_, Self>) -> PyResult<shakmaty::Chess> {
        let board = slf.borrow();
        let base_board = slf.as_super().borrow();

        let b = base_board.board()?;

        let setup = shakmaty::Setup {
            board: b,
            promoted: base_board.promoted,
            pockets: None,
            turn: board.turn,
            castling_rights: board.castling_rights,
            ep_square: board.ep_square,
            remaining_checks: None,
            halfmoves: board.halfmove_clock as u32,
            fullmoves: std::num::NonZeroU32::new(std::cmp::max(
                1,
                u32::from(board.fullmove_number.get()),
            ))
            .unwrap_or(std::num::NonZeroU32::MIN),
        };

        shakmaty::Chess::from_setup(setup, shakmaty::CastlingMode::Standard)
            .or_else(|e| e.ignore_too_much_material())
            .or_else(|e| e.ignore_impossible_check())
            .or_else(|e| e.ignore_invalid_castling_rights())
            .or_else(|e| e.ignore_invalid_ep_square())
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid state: {:?}", e)))
    }
}

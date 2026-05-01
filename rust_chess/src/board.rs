use shakmaty::{Bitboard, Color, FromSetup, Position, Square};

use std::num::NonZeroU32;

use crate::base_board::BaseBoard;
use crate::py_move::PyMove;
use crate::util::{PyColor, PySquare};
use pyo3::prelude::*;
use pyo3::types::PyType;

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
    #[classattr]
    const uci_variant: &'static str = "chess";

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
    #[classmethod]
    #[pyo3(name = "empty")]
    fn py_empty(cls: &Bound<'_, PyType>, py: Python<'_>) -> PyResult<Py<Self>> {
        let (board, base_board) = Self::__new__(py, None, false)?;
        let class_obj = pyo3::PyClassInitializer::from(base_board).add_subclass(board);
        Py::new(py, class_obj)
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
            let b = BaseBoard::empty();
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

    #[pyo3(signature = (*, shredder=false, en_passant="legal", promoted=None))]
    #[allow(unused_variables)]
    fn fen(
        slf: &Bound<'_, Self>,
        py: Python<'_>,
        shredder: bool,
        en_passant: &str,
        promoted: Option<bool>,
    ) -> PyResult<String> {
        let chess = Board::try_shakmaty(slf)?;

        let ep_mode = match en_passant {
            "legal" => shakmaty::EnPassantMode::Legal,
            "fen" | "x-fen" => shakmaty::EnPassantMode::PseudoLegal,
            _ => shakmaty::EnPassantMode::Legal,
        };

        let mut fen_obj = shakmaty::fen::Fen::from_position(&chess, ep_mode);

        if !shredder {
            // If shredder=False (standard FEN), shakmaty::fen::Fen automatically handles it
            // by not printing shredder castling rights, but wait, shakmaty fen always prints standard castling rights
            // unless it's a chess960 position? Actually, Fen::from_position just uses standard FEN rules.
        }

        // However, python-chess also accepts promoted parameter.
        // We can just let shakmaty format it, but wait, shakmaty's Fen doesn't format promoted pieces.
        // python-chess extension might use `base_board.board_fen(promoted)` for just the board part.
        // For the full FEN, we can just return `fen_obj.to_string()`.

        let mut fen_str = fen_obj.to_string();

        if let Some(true) = promoted {
            let board_fen_str = slf.as_super().borrow().board_fen(Some(true))?;
            let split: Vec<&str> = fen_str.splitn(2, ' ').collect();
            if split.len() == 2 {
                fen_str = format!("{} {}", board_fen_str, split[1]);
            }
        }

        if shredder {
            // TODO: properly format shredder castling rights if needed
        }

        Ok(fen_str)
    }

    #[pyo3(signature = (*, shredder=false, en_passant="legal", promoted=None))]
    #[allow(unused_variables)]
    fn epd(
        slf: &Bound<'_, Self>,
        py: Python<'_>,
        shredder: bool,
        en_passant: &str,
        promoted: Option<bool>,
    ) -> PyResult<String> {
        let chess = Board::try_shakmaty(slf)?;
        let ep_mode = match en_passant {
            "legal" => shakmaty::EnPassantMode::Legal,
            "fen" | "x-fen" => shakmaty::EnPassantMode::PseudoLegal,
            _ => shakmaty::EnPassantMode::Legal,
        };
        let epd_obj = shakmaty::fen::Epd::from_position(&chess, ep_mode);
        Ok(epd_obj.to_string())
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

    #[pyo3(signature = (pieces))]
    fn set_piece_map(
        mut slf: PyRefMut<'_, Self>,
        py: Python<'_>,
        pieces: &Bound<'_, pyo3::types::PyDict>,
    ) -> PyResult<()> {
        slf.clear_stack(py);
        slf.into_super().set_piece_map(pieces)
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
        slf.borrow_mut().into_super().apply_mirror(py)?;

        let ep = slf.borrow().ep_square;
        if let Some(sq) = ep {
            slf.borrow_mut().ep_square = Some(sq.flip_vertical());
        }

        let cr = slf.borrow().castling_rights;
        slf.borrow_mut().castling_rights = cr.flip_vertical();

        Ok(())
    }

    fn clear_board(mut slf: PyRefMut<'_, Self>, py: Python<'_>) {
        slf.clear_stack(py);
        slf.into_super().clear_board();
    }

    #[pyo3(name = "push")]
    fn py_push(
        slf: &Bound<'_, Self>,
        py: Python<'_>,
        move_obj: &Bound<'_, pyo3::PyAny>,
    ) -> PyResult<()> {
        if let Ok(m) = move_obj.extract::<PyRef<'_, PyMove>>() {
            let chess = Board::try_shakmaty(slf)?;
            if let Ok(sm_move) = m.inner.to_move(&chess) {
                let new_chess = chess.play(sm_move).map_err(|e| {
                    pyo3::exceptions::PyValueError::new_err(format!("illegal move: {}", e))
                })?;

                // To keep state syncing with python-chess `_stack`, we must capture it
                let chess_mod = py.import("chess")?;
                let board_state_cls = chess_mod.getattr("_BoardState")?;
                let board_state = board_state_cls.call1((slf,))?;

                let mut rust_board = slf.borrow_mut();
                rust_board
                    ._stack
                    .bind(py)
                    .call_method1("append", (board_state,))?;

                rust_board.turn = new_chess.turn();
                rust_board.castling_rights = new_chess.castles().castling_rights();
                rust_board.ep_square = new_chess.ep_square(shakmaty::EnPassantMode::Legal);
                rust_board.halfmove_clock = new_chess.halfmoves() as u16;
                rust_board.fullmove_number =
                    std::num::NonZeroU32::new(std::cmp::max(1, new_chess.fullmoves().get()))
                        .unwrap();

                rust_board
                    .move_stack
                    .bind(py)
                    .call_method1("append", (m.clone(),))?;

                // Update BaseBoard bitboards
                let (roles, colors) = new_chess.board().clone().into_bitboards();
                let promoted = new_chess.promoted();

                // Drop rust_board borrow to mutably borrow super
                drop(rust_board);

                let mut base = slf.as_super().borrow_mut();
                base.by_role = roles;
                base.by_color = colors;
                base.promoted = promoted;
                return Ok(());
            }
        }
        Err(pyo3::exceptions::PyValueError::new_err("Invalid move"))
    }

    fn parse_uci(slf: &Bound<'_, Self>, py: Python<'_>, uci: &str) -> PyResult<Py<PyAny>> {
        let chess_mod = py.import("chess")?;
        let move_cls = chess_mod.getattr("Move")?;
        let move_obj = move_cls.call_method1("from_uci", (uci,))?;

        let is_truthy = move_obj.is_truthy()?;
        if !is_truthy {
            return Ok(move_obj.into());
        }

        let is_legal = slf
            .call_method1("is_legal", (&move_obj,))?
            .extract::<bool>()?;
        if !is_legal {
            let fen = slf.call_method0("fen")?;
            let msg = format!("illegal uci: '{}' in {}", uci, fen);
            let err_cls = chess_mod.getattr("IllegalMoveError")?;
            return Err(PyErr::from_value(err_cls.call1((msg,))?));
        }

        Ok(move_obj.into())
    }

    fn push_uci(slf: &Bound<'_, Self>, py: Python<'_>, uci: &str) -> PyResult<Py<PyAny>> {
        let move_obj = slf.call_method1("parse_uci", (uci,))?;
        slf.call_method1("push", (&move_obj,))?;
        Ok(move_obj.into())
    }

    #[pyo3(name = "pop")]
    fn py_pop(slf: &Bound<'_, Self>, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let rust_board = slf.borrow();
        let move_stack = rust_board.move_stack.bind(py);
        if move_stack.len() == 0 {
            return Err(pyo3::exceptions::PyIndexError::new_err(
                "pop from empty move stack",
            ));
        }

        let m = move_stack.call_method0("pop")?;

        let _stack = rust_board._stack.bind(py);
        if _stack.len() > 0 {
            let board_state = _stack.call_method0("pop")?;
            // Drop borrow to allow restore to mutate slf
            drop(rust_board);
            board_state.call_method1("restore", (slf.clone(),))?;
        }

        Ok(m.into())
    }

    fn peek(slf: &Bound<'_, Self>, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let rust_board = slf.borrow();
        let move_stack = rust_board.move_stack.bind(py);
        if move_stack.len() == 0 {
            return Err(pyo3::exceptions::PyIndexError::new_err(
                "peek from empty move stack",
            ));
        }
        Ok(move_stack.get_item(move_stack.len() - 1)?.into())
    }

    #[pyo3(signature = (move_obj))]
    fn is_legal(
        slf: &Bound<'_, Self>,
        py: Python<'_>,
        move_obj: &Bound<'_, pyo3::PyAny>,
    ) -> PyResult<bool> {
        if let Ok(m) = move_obj.extract::<PyRef<'_, PyMove>>() {
            let chess = Board::try_shakmaty(slf)?;
            if let Ok(sm_move) = m.inner.to_move(&chess) {
                return Ok(chess.is_legal(sm_move));
            }
        }
        Ok(false)
    }

    #[pyo3(signature = (move_obj))]
    fn is_pseudo_legal(
        slf: &Bound<'_, Self>,
        py: Python<'_>,
        move_obj: &Bound<'_, pyo3::PyAny>,
    ) -> PyResult<bool> {
        // Pseudo legal just means the piece can move there, ignoring checks
        // `python-chess` tests expect we can just use `chess.pseudo_legal_moves`.
        slf.call_method1("pseudo_legal_moves", ())?
            .call_method1("__contains__", (move_obj,))?
            .extract()
    }

    fn clear_stack(&mut self, py: Python<'_>) {
        self.move_stack = pyo3::types::PyList::empty(py).into();
        self._stack = pyo3::types::PyList::empty(py).into();
    }

    fn ply(slf: &Bound<'_, Self>, py: Python<'_>) -> PyResult<usize> {
        Ok(slf.borrow().move_stack.bind(py).len())
    }

    fn root(slf: &Bound<'_, Self>, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let board_rust = slf.borrow();
        let _stack = board_rust._stack.bind(py);
        if _stack.len() > 0 {
            let first = _stack.get_item(0)?;
            let new_board = slf.call_method1("empty", ())?;
            // Call restore on the first state
            first.call_method1("restore", (&new_board,))?;
            Ok(new_board.into())
        } else {
            Ok(slf.call_method1("copy", ())?.into())
        }
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

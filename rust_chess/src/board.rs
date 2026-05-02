use pyo3::exceptions::PyValueError;
use shakmaty::uci::UciMove;
use shakmaty::{Bitboard, Color, FromSetup, Position, Square};

use std::num::NonZeroU32;
use std::str::FromStr;

use crate::base_board::BaseBoard;
use crate::py_move::PyMove;
use crate::util::{PyColor, PyRole, PySquare};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple, PyType};

#[pyclass(module = "rust_chess", name = "LegalMoveGenerator")]
pub struct LegalMoveGenerator {
    board: Py<Board>,
}

const ONE: NonZeroU32 = std::num::NonZeroU32::MIN;

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

#[derive(Clone, PartialEq, Eq)]
pub struct StateBoard {
    pub by_role: shakmaty::ByRole<Bitboard>,
    pub by_color: shakmaty::ByColor<Bitboard>,
    pub promoted: Bitboard,
    pub turn: Color,
    pub castling_rights: Bitboard,
    pub ep_square: Option<Square>,
    pub halfmove_clock: u16,
    pub fullmove_number: NonZeroU32,
}

impl StateBoard {
    // used for checking repetitions
    pub fn epd_tuple(
        &self,
    ) -> (
        shakmaty::ByRole<Bitboard>,
        shakmaty::ByColor<Bitboard>,
        Bitboard,
        Color,
        Bitboard,
        Option<Square>,
    ) {
        (
            self.by_role,
            self.by_color,
            self.promoted,
            self.turn,
            self.castling_rights,
            self.ep_square,
        )
    }
}

impl From<&Board> for StateBoard {
    fn from(board: &Board) -> Self {
        let (by_role, by_color) = shakmaty::Board::empty().into_bitboards();
        Self {
            by_role,
            by_color,
            promoted: Bitboard::EMPTY,
            turn: board.turn,
            castling_rights: board.castling_rights,
            ep_square: board.ep_square,
            halfmove_clock: board.halfmove_clock,
            fullmove_number: board.fullmove_number,
        }
    }
}

impl From<(&Board, &BaseBoard)> for StateBoard {
    fn from((board, base): (&Board, &BaseBoard)) -> Self {
        let mut state = StateBoard::from(board);
        state.by_role = base.by_role.clone();
        state.by_color = base.by_color.clone();
        state.promoted = base.promoted;
        state
    }
}

#[pyclass(extends=BaseBoard, subclass, dict)]
pub struct Board {
    pub turn: Color,
    pub castling_rights: Bitboard,
    pub ep_square: Option<Square>,
    pub halfmove_clock: u16,
    pub fullmove_number: NonZeroU32,
    pub move_stack: Vec<PyMove>,
    pub _stack: Vec<StateBoard>,
    pub chess960: bool,
}

#[pymethods]
impl Board {
    #[classattr]
    fn aliases() -> Vec<&'static str> {
        vec![
            "Standard",
            "Chess",
            "Classical",
            "Normal",
            "Illegal",
            "From Position",
        ]
    }

    #[classattr]
    #[allow(non_upper_case_globals)]
    const uci_variant: &'static str = "chess";

    #[classattr]
    #[allow(non_upper_case_globals)]
    const xboard_variant: &'static str = "normal";

    #[classattr]
    #[allow(non_upper_case_globals)]
    const starting_fen: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    #[getter]
    fn chess960(&self) -> bool {
        self.chess960
    }

    #[setter]
    fn set_chess960(&mut self, chess960: bool) {
        self.chess960 = chess960;
    }

    #[getter]
    fn move_stack(&self) -> Vec<PyMove> {
        self.move_stack.clone()
    }

    #[setter]
    fn set_move_stack(&mut self, stack: Vec<PyMove>) {
        self.move_stack = stack;
    }

    #[classmethod]
    #[pyo3(name = "empty")]
    fn py_empty(_cls: &Bound<'_, PyType>, py: Python<'_>) -> PyResult<Py<Self>> {
        let (board, base_board) = Self::empty();
        let class_obj = pyo3::PyClassInitializer::from(base_board).add_subclass(board);
        Py::new(py, class_obj)
    }

    #[new]
    #[pyo3(signature = (fen=Some("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"), *, chess960=false))]
    #[allow(unused_variables)]
    fn __new__(_py: Python<'_>, fen: Option<&str>, chess960: bool) -> PyResult<(Self, BaseBoard)> {
        let mut turn = Color::White;
        let mut castling_rights = Bitboard::EMPTY;
        let mut ep_square = None;
        let mut halfmove_clock = 0;
        let mut fullmove_number = ONE;

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
            fullmove_number,
            move_stack: Vec::new(),
            _stack: Vec::new(),
            chess960,
        };

        Ok((board, base_board))
    }

    #[pyo3(signature = (fen=Some("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"), *, chess960=false))]
    #[allow(unused_variables)]
    fn __init__(mut slf: PyRefMut<'_, Self>, fen: Option<&str>, chess960: bool) -> PyResult<()> {
        if let Some(f) = fen {
            let setup = shakmaty::fen::Fen::from_ascii(f.as_bytes())
                .map_err(|e| {
                    pyo3::exceptions::PyValueError::new_err(format!("invalid fen: {}", e))
                })?
                .into_setup();

            slf.turn = setup.turn;
            slf.castling_rights = setup.castling_rights;
            slf.ep_square = setup.ep_square;
            slf.halfmove_clock = setup.halfmoves as u16;
            slf.fullmove_number = NonZeroU32::new(setup.fullmoves.into()).unwrap_or(ONE);

            let (roles, colors) = setup.board.into_bitboards();
            let promoted = setup.promoted;
            slf.clear_stack();
            slf.chess960 = chess960;

            let mut base = slf.into_super();
            base.by_role = roles;
            base.by_color = colors;
            base.promoted = promoted;
        } else {
            slf.turn = Color::White;
            slf.castling_rights = Bitboard::EMPTY;
            slf.ep_square = None;
            slf.halfmove_clock = 0;
            slf.fullmove_number = NonZeroU32::new(1).unwrap();
            slf.clear_stack();
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
    fn set_fullmove_number(&mut self, fullmove_number: NonZeroU32) {
        self.fullmove_number = fullmove_number;
    }

    fn clear(mut slf: PyRefMut<'_, Self>) -> PyResult<()> {
        slf.turn = Color::White;
        slf.castling_rights = Bitboard::EMPTY;
        slf.ep_square = None;
        slf.halfmove_clock = 0;
        slf.fullmove_number = ONE;
        slf.clear_stack();

        let mut base = slf.into_super();
        (&mut *base).clear_board();
        Ok(())
    }

    fn reset(mut slf: PyRefMut<'_, Self>) -> PyResult<()> {
        slf.turn = Color::White;
        slf.castling_rights = Bitboard(0x8100_0000_0000_0081); // standard castling rights
        slf.ep_square = None;
        slf.halfmove_clock = 0;
        slf.fullmove_number = ONE;
        slf.clear_stack();

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
        slf.clear_stack();

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
        shredder: bool,
        en_passant: &str,
        promoted: Option<bool>,
    ) -> PyResult<String> {
        let _ = (shredder, en_passant);
        if let Some(true) = promoted {
            return Err(PyNotImplementedError::new_err("todo!"));
        }
        let chess = Self::try_shakmaty(slf)?;
        let fen = Fen::from_position(&chess, EnPassantMode::Legal);
        Ok(fen.to_string())
    }

    #[pyo3(signature = (*, en_passant="legal", promoted=None))]
    fn shredder_fen(
        slf: &Bound<'_, Self>,
        en_passant: &str,
        promoted: Option<bool>,
    ) -> PyResult<String> {
        let _ = en_passant;
        if shredder {
            return Err(PyNotImplementedError::new_err("todo!"));
        }
        if let Some(true) = promoted {
            return Err(PyNotImplementedError::new_err("todo!"));
        }
        let chess = Self::try_shakmaty(slf)?;
        let epd = Epd::from_position(&chess, EnPassantMode::Legal);
        Ok(epd.to_string())
    }

    #[pyo3(signature = (*, stack=None))]
    fn copy<'py>(
        slf: &Bound<'py, Self>,
        py: Python<'py>,
        stack: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, Self>> {
        let board_rust = slf.borrow();
        let base_rust = slf.as_super().borrow();

        let mut stack_len = board_rust.move_stack.len();
        if let Some(s) = stack {
            if let Ok(b) = s.extract::<bool>() {
                if !b {
                    stack_len = 0;
                }
            } else if let Ok(i) = s.extract::<usize>() {
                stack_len = i;
            }
        }

        let move_stack_start = board_rust.move_stack.len().saturating_sub(stack_len);
        let state_stack_start = board_rust._stack.len().saturating_sub(stack_len);

        let new_board = Board {
            turn: board_rust.turn,
            castling_rights: board_rust.castling_rights,
            ep_square: board_rust.ep_square,
            halfmove_clock: board_rust.halfmove_clock,
            fullmove_number: board_rust.fullmove_number,
            move_stack: board_rust.move_stack[move_stack_start..].to_vec(),
            _stack: board_rust._stack[state_stack_start..].to_vec(),
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
        _py: Python<'py>,
        memo: Bound<'py, PyAny>,
        stack: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, Self>> {
        let _ = memo;
        // deepcopy in python uses copy() essentially, we'll just do shallow for _stack.
        Self::copy(slf, slf.py(), stack)
    }

    #[getter]
    fn legal_moves<'py>(slf: &Bound<'py, Self>, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        todo!()
    }

    #[getter]
    fn pseudo_legal_moves<'py>(
        slf: &Bound<'py, Self>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let py_board = Self::to_python_board(slf, true)?;
        let chess_mod = py.import("chess")?;
        chess_mod
            .getattr("PseudoLegalMoveGenerator")?
            .call1((py_board,))
    }

    #[pyo3(signature = (from_mask=-1i128, to_mask=-1i128))]
    fn generate_pseudo_legal_moves<'py>(
        slf: &Bound<'py, Self>,
        _py: Python<'py>,
        from_mask: i128,
        to_mask: i128,
    ) -> PyResult<Bound<'py, pyo3::PyAny>> {
        let py_board = Self::to_python_board(slf, true)?;
        py_board.call_method1("generate_pseudo_legal_moves", (from_mask, to_mask))
    }

    #[pyo3(signature = (from_mask=-1i128, to_mask=-1i128))]
    fn generate_legal_moves<'py>(
        slf: &Bound<'py, Self>,
        from_mask: i128,
        to_mask: i128,
    ) -> PyResult<Bound<'py, pyo3::PyAny>> {
        // FIXME this is slow use arrayvec retain
        let mut py_moves = Vec::new();
        for m in chess.legal_moves() {
            let from_ok = m.from().is_none_or(|sq| from_mask.contains(sq));
            if from_ok && to_mask.contains(m.to()) {
                py_moves.push(m.into())
            }
        }
    }

    #[pyo3(signature = (from_mask=-1i128, to_mask=-1i128))]
    fn generate_castling_moves<'py>(
        slf: &Bound<'py, Self>,
        _py: Python<'py>,
        from_mask: i128,
        to_mask: i128,
    ) -> PyResult<Bound<'py, pyo3::PyAny>> {
        let py_board = Self::to_python_board(slf, true)?;
        py_board.call_method1("generate_castling_moves", (from_mask, to_mask))
    }

    #[pyo3(signature = (from_mask=-1i128, to_mask=-1i128))]
    fn generate_pseudo_legal_ep<'py>(
        slf: &Bound<'py, Self>,
        _py: Python<'py>,
        from_mask: i128,
        to_mask: i128,
    ) -> PyResult<Bound<'py, pyo3::PyAny>> {
        let py_board = Self::to_python_board(slf, true)?;
        py_board.call_method1("generate_pseudo_legal_ep", (from_mask, to_mask))
    }

    #[pyo3(signature = (from_mask=-1i128, to_mask=-1i128))]
    fn generate_legal_captures<'py>(
        slf: &Bound<'py, Self>,
        _py: Python<'py>,
        from_mask: i128,
        to_mask: i128,
    ) -> PyResult<Bound<'py, pyo3::PyAny>> {
        let py_board = Self::to_python_board(slf, true)?;
        py_board.call_method1("generate_legal_captures", (from_mask, to_mask))
    }

    #[pyo3(signature = (from_mask=-1i128, to_mask=-1i128))]
    fn generate_legal_ep<'py>(
        slf: &Bound<'py, Self>,
        from_mask: i128,
        to_mask: i128,
    ) -> PyResult<Bound<'py, pyo3::PyAny>> {
        let py_board = Self::to_python_board(slf, true)?;
        py_board.call_method1("generate_legal_ep", (from_mask, to_mask))
    }

    fn is_check(slf: &Bound<'_, Self>) -> PyResult<bool> {
        Ok(Self::try_shakmaty(slf)?.is_check())
    }

    fn is_variant_end(_slf: &Bound<'_, Self>) -> bool {
        false // not implemented for board
    }

    fn is_variant_win(_slf: &Bound<'_, Self>) -> bool {
        false // not implemented for board
    }

    fn is_variant_loss(_slf: &Bound<'_, Self>) -> bool {
        false // not implemented for board
    }

    fn is_variant_draw(_slf: &Bound<'_, Self>) -> bool {
        false // not implemented for board
    }

    #[pyo3(signature = (square, piece, promoted=false))]
    fn set_piece_at(
        mut slf: PyRefMut<'_, Self>,
        square: PySquare,
        piece: Option<crate::piece::PyPiece>,
        promoted: bool,
    ) {
        slf.clear_stack();
        slf.into_super().set_piece_at(square, piece, promoted);
    }

    fn remove_piece_at(
        mut slf: PyRefMut<'_, Self>,
        square: PySquare,
    ) -> Option<crate::piece::PyPiece> {
        slf.clear_stack();
        slf.into_super().remove_piece_at(square)
    }

    #[pyo3(signature = (pieces))]
    fn set_piece_map(
        mut slf: PyRefMut<'_, Self>,
        pieces: &Bound<'_, pyo3::types::PyDict>,
    ) -> PyResult<()> {
        slf.clear_stack();
        slf.into_super().set_piece_map(pieces)
    }

    fn set_board_fen(mut slf: PyRefMut<'_, Self>, fen: &str) -> PyResult<()> {
        slf.clear_stack();
        slf.into_super().set_board_fen(fen)
    }

    fn set_chess960_pos(mut slf: PyRefMut<'_, Self>, scharnagl: u16) -> PyResult<()> {
        todo!()
    }

    #[classmethod]
    fn from_chess960_pos(
        _cls: &Bound<'_, PyType>,
        py: Python<'_>,
        scharnagl: u16,
    ) -> PyResult<Py<Self>> {
        todo!()
    }

    fn apply_mirror(slf: &Bound<'_, Self>, py: Python<'_>) -> PyResult<()> {
        let turn = !slf.borrow().turn;
        slf.borrow_mut().turn = turn;
        let _ = py;
        slf.borrow_mut().into_super().apply_mirror()?;

        let ep = slf.borrow().ep_square;
        if let Some(sq) = ep {
            slf.borrow_mut().ep_square = Some(sq.flip_vertical());
        }

        let cr = slf.borrow().castling_rights;
        slf.borrow_mut().castling_rights = cr.flip_vertical();

        Ok(())
    }

    fn san(slf: &Bound<'_, Self>, move_obj: PyMove) -> PyResult<String> {
        let chess = Self::try_shakmaty(slf)?;
        let smove = move_obj
            .inner
            .to_move(&chess)
            .map_err(|_| PyValueError::new_err("illegal move"))?;
        Ok(shakmaty::san::San::from_move(&chess, smove).to_string())
    }

    fn lan(slf: &Bound<'_, Self>, move_obj: PyMove) -> PyResult<String> {
        Self::san(slf, move_obj)
    }

    fn variation_san(slf: &Bound<'_, Self>, variation: &Bound<'_, PyAny>) -> PyResult<String> {
        let mut chess = Self::try_shakmaty(slf)?;
        let mut out = String::new();
        let mut move_number = chess.fullmoves().get();
        let mut white_to_move = chess.turn().is_white();

        for item in variation.try_iter()? {
            let move_obj: PyMove = item?.extract()?;
            let smove = move_obj
                .inner
                .to_move(&chess)
                .map_err(|_| PyValueError::new_err("illegal move in variation"))?;

            let san = shakmaty::san::San::from_move(&chess, smove).to_string();

            if !out.is_empty() {
                out.push(' ');
            }
            if white_to_move {
                out.push_str(&format!("{}. {}", move_number, san));
            } else {
                out.push_str(&format!("{}... {}", move_number, san));
            }

            chess.play_unchecked(smove);
            if !white_to_move {
                move_number += 1;
            }
            white_to_move = !white_to_move;
        }

        Ok(out)
    }

    fn parse_san(slf: &Bound<'_, Self>, san: &str) -> PyResult<Py<PyAny>> {
        let py = slf.py();
        let chess = Self::try_shakmaty(slf)?;
        let parsed = shakmaty::san::San::from_str(san)
            .map_err(|e| PyValueError::new_err(format!("invalid san: {e}")))?;
        let smove = parsed
            .to_move(&chess)
            .map_err(|e| PyValueError::new_err(format!("illegal san: {e}")))?;
        let uci = smove.to_uci(shakmaty::CastlingMode::Standard);
        Ok(Bound::new(py, PyMove { inner: uci })?.into_any().unbind())
    }

    fn push_san(slf: &Bound<'_, Self>, san: &str) -> PyResult<Py<PyAny>> {
        let move_obj = slf.call_method1("parse_san", (san,))?;
        slf.call_method1("push", (&move_obj,))?;
        Ok(move_obj.into())
    }

    fn parse_xboard(slf: &Bound<'_, Self>, xboard: &str) -> PyResult<Py<PyAny>> {
        Self::parse_san(slf, xboard)
    }

    fn push_xboard(slf: &Bound<'_, Self>, xboard: &str) -> PyResult<Py<PyAny>> {
        Self::push_san(slf, xboard)
    }

    #[pyo3(signature = (move_obj, chess960=None))]
    fn uci(slf: &Bound<'_, Self>, move_obj: PyMove, chess960: Option<bool>) -> PyResult<String> {
        let chess = Self::try_shakmaty(slf)?;
        let smove = move_obj
            .inner
            .to_move(&chess)
            .map_err(|_| PyValueError::new_err("illegal move"))?;
        let mode = if chess960.unwrap_or(slf.borrow().chess960) {
            shakmaty::CastlingMode::Chess960
        } else {
            shakmaty::CastlingMode::Standard
        };
        Ok(smove.to_uci(mode).to_string())
    }

    fn xboard(slf: &Bound<'_, Self>, move_obj: PyMove) -> PyResult<String> {
        let chess = Self::try_shakmaty(slf)?;
        let smove = move_obj
            .inner
            .to_move(&chess)
            .map_err(|_| PyValueError::new_err("illegal move"))?;
        if smove.castling_side().is_some() {
            if smove.castling_side() == Some(shakmaty::CastlingSide::KingSide) {
                Ok("O-O".to_string())
            } else {
                Ok("O-O-O".to_string())
            }
        } else {
            Ok(match move_obj.inner {
                UciMove::Null => "@@@@".to_string(),
                _ => move_obj.inner.to_string(),
            })
        }
    }

    fn is_capture(slf: &Bound<'_, Self>, move_obj: PyMove) -> PyResult<bool> {
        let chess = Self::try_shakmaty(slf)?;
        let smove = move_obj
            .inner
            .to_move(&chess)
            .map_err(|_| PyValueError::new_err("illegal move"))?;
        Ok(smove.is_capture())
    }

    fn is_castling(slf: &Bound<'_, Self>, move_obj: PyMove) -> PyResult<bool> {
        let chess = Self::try_shakmaty(slf)?;
        let smove = move_obj
            .inner
            .to_move(&chess)
            .map_err(|_| PyValueError::new_err("illegal move"))?;
        Ok(smove.is_castle())
    }

    fn is_irreversible(slf: &Bound<'_, Self>, move_obj: PyMove) -> PyResult<bool> {
        let chess = Self::try_shakmaty(slf)?;
        let smove = move_obj
            .inner
            .to_move(&chess)
            .map_err(|_| PyValueError::new_err("illegal move"))?;
        Ok(chess.is_irreversible(smove))
    }

    #[pyo3(signature = (from_square, to_square, promotion=None))]
    fn find_move(
        slf: &Bound<'_, Self>,
        from_square: PySquare,
        to_square: PySquare,
        promotion: Option<PyRole>,
    ) -> PyResult<PyMove> {
        let chess = Self::try_shakmaty(slf)?;
        let wanted_promotion = promotion.map(|r| r.0);

        for m in chess.legal_moves() {
            if m.from() == Some(from_square.0)
                && m.to() == to_square.0
                && m.promotion() == wanted_promotion
            {
                return Ok(PyMove {
                    inner: m.to_uci(shakmaty::CastlingMode::Standard),
                });
            }
        }

        Err(PyValueError::new_err("no matching legal move found"))
    }

    fn clean_castling_rights(slf: &Bound<'_, Self>) -> PyResult<u64> {
        let chess = Self::try_shakmaty(slf)?;
        Ok(chess.castles().castling_rights().0)
    }

    fn has_kingside_castling_rights(slf: &Bound<'_, Self>, color: PyColor) -> PyResult<bool> {
        let chess = Self::try_shakmaty(slf)?;
        Ok(chess
            .castles()
            .has(color.0, shakmaty::CastlingSide::KingSide))
    }

    fn has_insufficient_material(slf: &Bound<'_, Self>, color: PyColor) -> PyResult<bool> {
        let chess = Self::try_shakmaty(slf)?;
        Ok(chess.has_insufficient_material(color.0))
    }

    fn status(slf: &Bound<'_, Self>) -> PyResult<u32> {
        let _ = slf;
        Ok(0)
    }

    fn is_valid(slf: &Bound<'_, Self>) -> PyResult<bool> {
        Ok(Self::try_shakmaty(slf).is_ok())
    }

    fn is_fifty_moves(slf: &Bound<'_, Self>) -> PyResult<bool> {
        let chess = Self::try_shakmaty(slf)?;
        Ok(chess.halfmoves() >= 100)
    }

    #[pyo3(signature = (count=3))]
    fn is_repetition(slf: &Bound<'_, Self>, count: usize) -> PyResult<bool> {
        let _ = (slf, count);
        Ok(false)
    }

    fn can_claim_threefold_repetition(slf: &Bound<'_, Self>) -> PyResult<bool> {
        let _ = slf;
        Ok(false)
    }

    #[pyo3(signature = (*, claim_draw=false))]
    fn result(slf: &Bound<'_, Self>, claim_draw: bool) -> PyResult<String> {
        let chess = Self::try_shakmaty(slf)?;
        let _ = claim_draw;
        if !chess.is_game_over() {
            return Ok("*".to_string());
        }
        match chess.outcome().winner() {
            Some(Color::White) => Ok("1-0".to_string()),
            Some(Color::Black) => Ok("0-1".to_string()),
            None => Ok("1/2-1/2".to_string()),
        }
    }

    #[pyo3(signature = (epd))]
    fn set_epd(slf: &Bound<'_, Self>, epd: &str) -> PyResult<Py<PyAny>> {
        let py = slf.py();
        let parsed = shakmaty::fen::Epd::from_str(epd)
            .map_err(|e| PyValueError::new_err(format!("invalid epd: {e}")))?;
        let chess = parsed
            .into_position::<shakmaty::Chess>(shakmaty::CastlingMode::Standard)
            .map_err(|e| PyValueError::new_err(format!("invalid epd position: {e}")))?;
        Self::from_chess_but_stack(slf, &chess);
        Ok(PyDict::new(py).into_any().unbind())
    }

    #[classmethod]
    #[pyo3(signature = (epd, *, chess960=false))]
    fn from_epd(
        _cls: &Bound<'_, PyType>,
        py: Python<'_>,
        epd: &str,
        chess960: bool,
    ) -> PyResult<Py<PyAny>> {
        let parsed = shakmaty::fen::Epd::from_str(epd)
            .map_err(|e| PyValueError::new_err(format!("invalid epd: {e}")))?;
        let chess = parsed
            .into_position::<shakmaty::Chess>(shakmaty::CastlingMode::Standard)
            .map_err(|e| PyValueError::new_err(format!("invalid epd position: {e}")))?;

        let (mut board, mut base) = Self::empty();
        board.chess960 = chess960;
        {
            let (roles, colors) = chess.board().clone().into_bitboards();
            base.by_role = roles;
            base.by_color = colors;
            base.promoted = chess.promoted();
            board.turn = chess.turn();
            board.castling_rights = chess.castles().castling_rights();
            board.ep_square = chess.ep_square(shakmaty::EnPassantMode::Legal);
            board.halfmove_clock = chess.halfmoves() as u16;
            board.fullmove_number = chess.fullmoves();
        }

        let board = Bound::new(py, (board, base))?;
        let tuple = PyTuple::new(py, [board.into_any(), PyDict::new(py).into_any()])?;
        Ok(tuple.into_any().unbind())
    }

    #[pyo3(signature = (*, invert_color=false, borders=false, empty_square="\u{2b58}", orientation=true))]
    fn unicode(
        slf: &Bound<'_, Self>,
        invert_color: bool,
        borders: bool,
        empty_square: &str,
        orientation: bool,
    ) -> PyResult<String> {
        todo!()
    }

    fn is_checkmate(slf: &Bound<'_, Self>) -> PyResult<bool> {
        Ok(Self::try_shakmaty(slf)?.is_checkmate())
    }

    fn __int__(slf: &Bound<'_, Self>) -> PyResult<u64> {
        todo!()
    }

    fn clear_board(mut slf: PyRefMut<'_, Self>) {
        slf.clear_stack();
        slf.into_super().clear_board();
    }

    #[pyo3(name = "push")]
    fn py_push(slf: &Bound<'_, Self>, move_obj: PyMove) -> PyResult<()> {
        let chess = Self::try_shakmaty(slf)?;
        let board_state = {
            let rust_board = slf.borrow();
            let base_board = slf.as_super().borrow();
            StateBoard::from((&*rust_board, &*base_board))
        };

        let sm_move = move_obj
            .inner
            .to_move(&chess)
            .map_err(|e| PyValueError::new_err(format!("Invalid move: {e}")))?;
        let new_chess = chess
            .play(sm_move)
            .map_err(|e| PyValueError::new_err(format!("illegal move: {e}")))?;

        {
            let mut rust_board = slf.borrow_mut();
            rust_board.move_stack.push(move_obj);
            rust_board._stack.push(board_state);
        }

        Self::from_chess_but_stack(slf, &new_chess);
        Ok(())
    }

    fn parse_uci(slf: &Bound<'_, Self>, _py: Python<'_>, uci: &str) -> PyResult<PyMove> {
        let inner = UciMove::from_str(uci)
            .map_err(|_| PyValueError::new_err(format!("invalid uci: {uci:?}")))?;

        if !matches!(inner, UciMove::Null) {
            let chess = Self::try_shakmaty(slf)?;
            let smove = inner
                .to_move(&chess)
                .map_err(|_| PyValueError::new_err(format!("illegal uci: {uci:?}")))?;
            if !chess.is_legal(smove) {
                return Err(PyValueError::new_err(format!("illegal uci: {uci:?}")));
            }
        }

        Ok(PyMove { inner })
    }

    fn push_uci(slf: &Bound<'_, Self>, uci: &str) -> PyResult<Py<PyAny>> {
        let move_obj = slf.call_method1("parse_uci", (uci,))?;
        slf.call_method1("push", (&move_obj,))?;
        Ok(move_obj.into())
    }

    #[pyo3(name = "pop")]
    fn py_pop(slf: &Bound<'_, Self>, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let (m, board_state) = {
            let mut rust_board = slf.borrow_mut();
            let m = rust_board.move_stack.pop().ok_or_else(|| {
                pyo3::exceptions::PyIndexError::new_err("pop from empty move stack")
            })?;
            let board_state = rust_board._stack.pop();
            (m, board_state)
        };

        if let Some(state) = board_state {
            Self::from_stateboard_but_stack(slf, &state);
        }

        Ok(Bound::new(py, m)?.into_any().unbind())
    }

    fn peek(slf: &Bound<'_, Self>, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let move_obj = {
            let rust_board = slf.borrow();
            rust_board.move_stack.last().cloned().ok_or_else(|| {
                pyo3::exceptions::PyIndexError::new_err("peek from empty move stack")
            })?
        };
        Ok(Bound::new(py, move_obj)?.into_any().unbind())
    }

    fn is_legal(slf: &Bound<'_, Self>, move_obj: PyMove) -> PyResult<bool> {
        let chess = Board::try_shakmaty(slf)?;
        Ok(move_obj
            .inner
            .to_move(&chess)
            .map(|m| chess.is_legal(m))
            .unwrap_or_default())
    }

    #[pyo3(signature = (move_obj))]
    fn is_pseudo_legal(slf: &Bound<'_, Self>, move_obj: PyMove) -> PyResult<bool> {
        todo!()
    }

    fn clear_stack(&mut self) {
        self.move_stack.clear();
        self._stack.clear();
    }

    fn ply(&self) -> usize {
        let fullmoves = self.fullmove_number.get() as usize;
        2 * (fullmoves.saturating_sub(1)) + usize::from(self.turn == Color::Black)
    }

    fn root<'py>(slf: &Bound<'py, Self>, py: Python<'py>) -> PyResult<Bound<'py, Self>> {
        let root = Bound::new(py, Self::empty())?;

        let first_state = {
            let rust_board = slf.borrow();
            rust_board._stack.first().cloned()
        };

        if let Some(state) = first_state {
            Self::from_stateboard_but_stack(&root, &state);
        } else {
            let chess = Self::try_shakmaty(slf)?;
            Self::from_chess_but_stack(&root, &chess);
        }

        Ok(root)
    }

    fn mirror(slf: &Bound<'_, Self>) -> PyResult<Py<PyAny>> {
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
    fn from_chess_but_stack(slf: &Bound<'_, Self>, chess: &shakmaty::Chess) {
        {
            let mut rust_board = slf.borrow_mut();
            rust_board.turn = chess.turn();
            rust_board.castling_rights = chess.castles().castling_rights();
            rust_board.ep_square = chess.ep_square(shakmaty::EnPassantMode::Legal);
            rust_board.halfmove_clock = chess.halfmoves() as u16;
            rust_board.fullmove_number = chess.fullmoves();
        }

        let (roles, colors) = chess.board().clone().into_bitboards();
        let promoted = chess.promoted();

        let mut base = slf.as_super().borrow_mut();
        base.by_role = roles;
        base.by_color = colors;
        base.promoted = promoted;
    }

    fn from_stateboard_but_stack(slf: &Bound<'_, Self>, state: &StateBoard) {
        {
            let mut rust_board = slf.borrow_mut();
            rust_board.turn = state.turn;
            rust_board.castling_rights = state.castling_rights;
            rust_board.ep_square = state.ep_square;
            rust_board.halfmove_clock = state.halfmove_clock;
            rust_board.fullmove_number = state.fullmove_number;
        }

        let mut base = slf.as_super().borrow_mut();
        base.by_role = state.by_role.clone();
        base.by_color = state.by_color.clone();
        base.promoted = state.promoted;
    }

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
            fullmoves: board.fullmove_number,
        };

        shakmaty::Chess::from_setup(setup, shakmaty::CastlingMode::Standard)
            .or_else(|e| e.ignore_too_much_material())
            .or_else(|e| e.ignore_impossible_check())
            .or_else(|e| e.ignore_invalid_castling_rights())
            .or_else(|e| e.ignore_invalid_ep_square())
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid state: {:?}", e)))
    }

    fn empty() -> (Self, BaseBoard) {
        let turn = Color::White;
        let castling_rights = Bitboard::EMPTY;
        let ep_square = None;
        let halfmove_clock = 0;
        let fullmove_number = ONE;
        (
            Self {
                turn,
                castling_rights,
                ep_square,
                halfmove_clock,
                fullmove_number,
                move_stack: Vec::new(),
                _stack: Vec::new(),
                chess960: false,
            },
            BaseBoard::empty(),
        )
    }
}

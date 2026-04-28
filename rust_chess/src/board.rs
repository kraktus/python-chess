use pyo3::ffi::PyObject;
use shakmaty::{Bitboard, Color, FromSetup, Position, Square};

use std::num::NonZeroU32;

use crate::base_board::BaseBoard;
use crate::py_move::PyMove;
use crate::util::{PyColor, PySquare};
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

pub struct BoardState {
    pub castling_rights: Bitboard,
    pub ep_square: Option<Square>,
    pub halfmove_clock: u16,
    pub fullmove_number: NonZeroU32,
}

#[pyclass(extends=BaseBoard, subclass, dict)]
pub struct Board {
    pub turn: Color,
    pub castling_rights: Bitboard,
    pub ep_square: Option<Square>,
    pub halfmove_clock: u16,
    pub fullmove_number: NonZeroU32,
    pub move_stack: Vec<PyMove>,
    pub _stack: Vec<BoardState>,
}

#[pymethods]
impl Board {
    #[new]
    #[pyo3(signature = (fen=Some("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"), *, chess960=false))]
    #[allow(unused_variables)]
    fn __new__(fen: Option<&str>, chess960: bool) -> PyResult<(Self, BaseBoard)> {
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
            move_stack: Vec::new(),
            _stack: Vec::new(),
        };

        Ok((board, base_board))
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

    fn clear(mut slf: PyRefMut<'_, Self>) {
        slf.turn = shakmaty::Color::White;
        slf.castling_rights = shakmaty::Bitboard::EMPTY;
        slf.ep_square = None;
        slf.halfmove_clock = 0;
        slf.fullmove_number = NonZeroU32::new(1).unwrap();
        slf.move_stack.clear();
        slf._stack.clear();

        let mut base = slf.into_super();
        (&mut *base).clear_board();
    }

    fn reset(mut slf: PyRefMut<'_, Self>) {
        slf.turn = shakmaty::Color::White;
        slf.castling_rights = shakmaty::Bitboard(0x8100_0000_0000_0081); // standard castling rights
        slf.ep_square = None;
        slf.halfmove_clock = 0;
        slf.fullmove_number = NonZeroU32::new(1).unwrap();
        slf.move_stack.clear();
        slf._stack.clear();

        let mut base = slf.into_super();
        (&mut *base).reset_board();
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
        slf.move_stack.clear();
        slf._stack.clear();

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

    fn is_game_over(slf: &Bound<'_, Self>, _claim_draw: Option<bool>) -> PyResult<bool> {
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
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid state: {:?}", e)))
    }
}

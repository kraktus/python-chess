#![allow(unused_variables)]
use pyo3::exceptions::PyValueError;
use shakmaty::fen::Fen;
use shakmaty::san::{San, SanError, SanPlus};
use shakmaty::uci::UciMove;
use shakmaty::{
    Bitboard, Castles, CastlingMode, CastlingSide, Chess, Color, File, FromSetup, Move, MoveList,
    Position, PseudoLegal, Role, Setup, Square,
};

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::str::FromStr;

use crate::base_board::BaseBoard;
use crate::py_move::PyMove;
use crate::util::{IntOrBool, PyColor, PyRole, PySquare};
use crate::{AmbiguousMoveError, IllegalMoveError, InvalidMoveError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple, PyType};

#[pyclass(module = "rust_chess", name = "LegalMoveGeneratorIter")]
pub struct LegalMoveGeneratorIter {
    moves: std::vec::IntoIter<PyMove>,
}

#[pymethods]
impl LegalMoveGeneratorIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyMove> {
        slf.moves.next()
    }
}

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
        let chess = Board::try_shakmaty(board)?;
        Ok(!chess.legal_moves().is_empty())
    }

    fn __len__(&self, py: Python<'_>) -> PyResult<usize> {
        self.count(py)
    }

    fn count(&self, py: Python<'_>) -> PyResult<usize> {
        let board = self.board.bind(py);
        let chess = Board::try_shakmaty(board)?;
        Ok(chess.legal_moves().len())
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<LegalMoveGeneratorIter> {
        let board = self.board.bind(py);
        let moves = Board::generate_legal_moves(board, Bitboard::FULL.0, Bitboard::FULL.0)?;
        Ok(LegalMoveGeneratorIter {
            moves: moves.into_iter(),
        })
    }

    fn __contains__(&self, move_obj: PyMove, py: Python<'_>) -> PyResult<bool> {
        let board = self.board.bind(py);
        let chess = Board::try_shakmaty(board)?;
        Ok(move_obj
            .inner
            .to_move(&chess)
            .is_ok_and(|m| chess.is_legal(m)))
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        let py = slf.py();
        let self_rust = slf.borrow();
        let board = self_rust.board.bind(py);
        let chess = Board::try_shakmaty(board)?;
        let moves = chess.legal_moves();
        let mut sans = Vec::new();
        for m in &moves {
            sans.push(shakmaty::san::SanPlus::from_move(chess.clone(), *m).to_string());
        }
        Ok(format!(
            "<LegalMoveGenerator at {:#x} ({})>",
            slf.as_ptr() as usize,
            sans.join(", ")
        ))
    }
}

pub type TranspositionKey = (
    shakmaty::ByRole<Bitboard>,
    shakmaty::ByColor<Bitboard>,
    Bitboard,
    Color,
    Bitboard,
    Option<Square>,
);

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
    #[must_use]
    fn transposition_key(&self) -> TranspositionKey {
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

impl From<(&Board, &BaseBoard)> for StateBoard {
    fn from((board, base): (&Board, &BaseBoard)) -> Self {
        let by_role = base.by_role;
        let by_color = base.by_color;
        let promoted = base.promoted;
        Self {
            by_role,
            by_color,
            promoted,
            turn: board.turn,
            castling_rights: board.castling_rights,
            ep_square: board.ep_square,
            halfmove_clock: board.halfmove_clock,
            fullmove_number: board.fullmove_number,
        }
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
            let setup = Fen::from_ascii(f.as_bytes())
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("invalid fen: {e}")))?
                .into_setup();

            turn = setup.turn;
            castling_rights = setup.castling_rights;
            ep_square = setup.ep_square;
            halfmove_clock = setup.halfmoves as u16;
            fullmove_number = setup.fullmoves;

            let (roles, colors) = setup.board.into_bitboards();
            BaseBoard {
                by_role: roles,
                by_color: colors,
                promoted: setup.promoted,
            }
        } else {
            BaseBoard::empty()
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
            let setup = Fen::from_ascii(f.as_bytes())
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("invalid fen: {e}")))?
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
        self.turn = turn.0;
    }

    #[getter]
    fn castling_rights(&self) -> u64 {
        self.castling_rights.0
    }

    #[setter]
    fn set_castling_rights(&mut self, castling_rights: u64) {
        self.castling_rights = shakmaty::Bitboard(castling_rights);
    }

    fn set_castling_fen(slf: &Bound<'_, Self>, castling_fen: &str) -> PyResult<()> {
        slf.borrow_mut().clear_stack();
        let mut setup = Self::try_setup(slf)?;
        setup.castling_rights = Bitboard::EMPTY;
        // copied from shakmaty FEN
        // TODO, upstream as separate method
        let sq_iter = castling_fen.chars().map(|ch| {
            let color = Color::from_white(ch.is_ascii_uppercase());
            let rooks_and_kings = setup.board.by_color(color)
                & (setup.board.rooks() | setup.board.kings())
                & color.backrank();
            let sq: PyResult<Square> = Ok(match ch.to_ascii_lowercase() {
                'k' => rooks_and_kings
                    .last()
                    .filter(|sq| setup.board.rooks().contains(*sq))
                    .unwrap_or_else(|| Square::from_coords(File::H, color.backrank())),
                'q' => rooks_and_kings
                    .first()
                    .filter(|sq| setup.board.rooks().contains(*sq))
                    .unwrap_or_else(|| Square::from_coords(File::A, color.backrank())),
                file => Square::from_coords(
                    File::from_char(char::from(file)).ok_or_else(|| {
                        PyValueError::new_err(format!(
                            "invalid castling fen: invalid file '{file}'"
                        ))
                    })?,
                    color.backrank(),
                ),
            });
            sq
        });
        for sq in sq_iter {
            setup.castling_rights |= sq?;
        }
        Self::mut_from_setup_but_stack_fullmove(slf, &setup);

        Ok(())
    }

    #[getter]
    fn ep_square(&self) -> Option<u32> {
        self.ep_square.map(u32::from)
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
        base.clear_board();
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
        base.reset_board();
        Ok(())
    }

    fn set_fen(mut slf: PyRefMut<'_, Self>, fen: &str) -> PyResult<()> {
        let setup = Fen::from_ascii(fen.as_bytes())
            .map_err(|e| PyValueError::new_err(format!("invalid fen: {e}")))?
            .into_setup();

        slf.turn = setup.turn;
        slf.castling_rights = setup.castling_rights;
        slf.ep_square = setup.ep_square;
        slf.halfmove_clock = setup.halfmoves as u16;
        slf.fullmove_number = setup.fullmoves;
        slf.clear_stack();

        let mut base = slf.into_super();
        let (roles, colors) = setup.board.into_bitboards();
        base.by_role = roles;
        base.by_color = colors;
        base.promoted = setup.promoted;

        Ok(())
    }

    #[pyo3(signature = (*, shredder=false, en_passant="legal", promoted=None))]
    fn fen(
        slf: &Bound<'_, Self>,
        shredder: bool,
        en_passant: &str,
        promoted: Option<bool>,
    ) -> PyResult<String> {
        let board = slf.borrow();
        let chess = Self::try_shakmaty_with_promoted(slf, promoted.unwrap_or_default())?;
        let setup = chess.clone().to_setup(match en_passant {
            "legal" => shakmaty::EnPassantMode::Legal,
            "xfen" => shakmaty::EnPassantMode::PseudoLegal,
            // fen mode
            _ => shakmaty::EnPassantMode::Always,
        });

        let fen = Fen::try_from_setup(setup)
            .map_err(|e| PyValueError::new_err(format!("unable to gen FEN: {e:?}")))?;
        Ok(if shredder {
            fen.to_string_with_shredder()
        } else {
            fen.to_string()
        })
    }

    #[pyo3(signature = (*, shredder=false, en_passant="legal", promoted=None, **operations))]
    fn epd(
        slf: &Bound<'_, Self>,
        shredder: bool,
        en_passant: &str,
        promoted: Option<bool>,
        operations: Option<Bound<'_, PyDict>>,
    ) -> PyResult<String> {
        let fen = Self::fen(slf, shredder, en_passant, promoted)?;
        let mut parts = fen.split_whitespace();
        let board_part = parts.next().unwrap_or_default();
        let turn_part = parts.next().unwrap_or_default();
        let castling_part = parts.next().unwrap_or_default();
        let ep_part = parts.next().unwrap_or_default();

        let mut epd = format!("{board_part} {turn_part} {castling_part} {ep_part}");
        let operations = crate::epd_ops::py_to_epd_operations(slf, operations.as_ref())?;
        let ops = crate::epd_ops::format_epd_operations(slf, &operations)?;
        if !ops.is_empty() {
            epd.push(' ');
            epd.push_str(&ops);
        }
        Ok(epd)
    }

    #[pyo3(signature = (*, en_passant="legal", promoted=None))]
    fn shredder_fen(
        slf: &Bound<'_, Self>,
        en_passant: &str,
        promoted: Option<bool>,
    ) -> PyResult<String> {
        Self::fen(slf, true, en_passant, promoted)
    }

    #[pyo3(signature = (*, stack=IntOrBool::Bool(true)))]
    fn copy<'py>(
        slf: &Bound<'py, Self>,
        py: Python<'py>,
        // stack can be option<int|bool>
        stack: IntOrBool,
    ) -> PyResult<Bound<'py, Self>> {
        let board = slf.borrow();
        let base_board = slf.as_super().borrow();

        assert_eq!(board.move_stack.len(), board._stack.len());
        let stack_len = stack.stack_len(board.move_stack.len());

        let move_stack_start = board.move_stack.len().saturating_sub(stack_len);
        let state_stack_start = board._stack.len().saturating_sub(stack_len);

        let new_board = Board {
            turn: board.turn,
            castling_rights: board.castling_rights,
            ep_square: board.ep_square,
            halfmove_clock: board.halfmove_clock,
            fullmove_number: board.fullmove_number,
            move_stack: board.move_stack[move_stack_start..].to_vec(),
            _stack: board._stack[state_stack_start..].to_vec(),
            chess960: board.chess960,
        };
        let new_base = base_board.clone();

        Bound::new(py, (new_board, new_base))
    }

    #[getter]
    fn legal_moves<'py>(slf: &Bound<'py, Self>, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let generator = LegalMoveGenerator::py_new(slf.clone().unbind());
        Ok(Bound::new(py, generator)?.into_any())
    }

    #[getter]
    fn pseudo_legal_moves<'py>(
        slf: &Bound<'py, Self>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let generator = PseudoLegalMoveGenerator::py_new(slf.clone().unbind());
        Ok(Bound::new(py, generator)?.into_any())
    }

    #[pyo3(signature = (from_mask=Bitboard::FULL.0, to_mask=Bitboard::FULL.0))]
    fn generate_pseudo_legal_moves(
        slf: &Bound<'_, Self>,
        from_mask: u64,
        to_mask: u64,
    ) -> PyResult<Vec<PyMove>> {
        Self::gen_pseudo_moves_and_filter(slf, from_mask, to_mask, |_| true)
    }

    #[pyo3(signature = (from_mask=Bitboard::FULL.0, to_mask=Bitboard::FULL.0))]
    fn generate_legal_moves(
        slf: &Bound<'_, Self>,
        from_mask: u64,
        to_mask: u64,
    ) -> PyResult<Vec<PyMove>> {
        Self::gen_legal_moves_and_filter(slf, from_mask, to_mask, |_| true)
    }

    #[pyo3(signature = (from_mask=Bitboard::FULL.0, to_mask=Bitboard::FULL.0))]
    fn generate_castling_moves(
        slf: &Bound<'_, Self>,
        from_mask: u64,
        to_mask: u64,
    ) -> PyResult<Vec<PyMove>> {
        Self::gen_legal_moves_and_filter(slf, from_mask, to_mask, |m| m.is_castle())
    }

    #[pyo3(signature = (from_mask=Bitboard::FULL.0, to_mask=Bitboard::FULL.0))]
    fn generate_pseudo_legal_ep(
        slf: &Bound<'_, Self>,
        from_mask: u64,
        to_mask: u64,
    ) -> PyResult<Vec<PyMove>> {
        Self::gen_pseudo_moves_and_filter(slf, from_mask, to_mask, |m| m.is_en_passant())
    }

    #[pyo3(signature = (from_mask=Bitboard::FULL.0, to_mask=Bitboard::FULL.0))]
    fn generate_legal_captures(
        slf: &Bound<'_, Self>,
        from_mask: u64,
        to_mask: u64,
    ) -> PyResult<Vec<PyMove>> {
        Self::gen_legal_moves_and_filter(slf, from_mask, to_mask, |m| m.is_capture())
    }

    #[pyo3(signature = (from_mask=Bitboard::FULL.0, to_mask=Bitboard::FULL.0))]
    fn generate_legal_ep(
        slf: &Bound<'_, Self>,
        from_mask: u64,
        to_mask: u64,
    ) -> PyResult<Vec<PyMove>> {
        Self::gen_legal_moves_and_filter(slf, from_mask, to_mask, |m| m.is_en_passant())
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

    #[pyo3(name = "set_chess960_pos")]
    fn py_set_chess960_pos(mut slf: PyRefMut<'_, Self>, scharnagl: u32) -> PyResult<()> {
        slf.clear_stack();
        slf.as_super().set_chess960_pos(scharnagl)?;
        slf.chess960 = true;
        slf.turn = Color::White;
        slf.castling_rights = slf.as_super().rooks();
        slf.ep_square = None;
        slf.halfmove_clock = 0;
        slf.fullmove_number = ONE;
        Ok(())
    }

    #[pyo3(signature = (*, ignore_turn=false, ignore_castling=false, ignore_counters=true))]
    fn chess960_pos(
        slf: &Bound<'_, Self>,
        ignore_turn: bool,
        ignore_castling: bool,
        ignore_counters: bool,
    ) -> Option<u32> {
        let board = slf.borrow();
        if board.ep_square.is_some() {
            return None;
        }

        if !ignore_turn && board.turn != Color::White {
            return None;
        }

        if !ignore_castling && board.castling_rights != slf.borrow().as_super().rooks() {
            return None;
        }

        if !ignore_counters && (board.fullmove_number != ONE || board.halfmove_clock != 0) {
            return None;
        }

        slf.borrow().as_super().chess960_pos()
    }

    #[classmethod]
    fn from_chess960_pos(
        _cls: &Bound<'_, PyType>,
        py: Python<'_>,
        scharnagl: u32,
    ) -> PyResult<Py<Self>> {
        let (mut board, mut base_board) = Self::empty();
        base_board.set_chess960_pos(scharnagl)?;
        board.chess960 = true;
        board.turn = Color::White;
        board.castling_rights = base_board.rooks();
        board.ep_square = None;
        board.halfmove_clock = 0;
        board.fullmove_number = ONE;
        let class_obj = pyo3::PyClassInitializer::from(base_board).add_subclass(board);
        Py::new(py, class_obj)
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
        Ok(SanPlus::from_move(chess, smove).to_string())
    }

    fn lan(slf: &Bound<'_, Self>, move_obj: PyMove) -> PyResult<String> {
        if move_obj.inner.is_null() {
            return Ok(move_obj.uci());
        }
        let chess = Self::try_shakmaty(slf)?;
        let smove = move_obj.inner.to_move(&chess).map_err(|_| {
            PyValueError::new_err(format!(
                "illegal move {move_obj:?} in position {}",
                Fen::from_position(&chess, shakmaty::EnPassantMode::Always)
            ))
        })?;
        let san = SanPlus::from_move(chess.clone(), smove);
        let san_str = san.to_string();
        if let Some(from_sq) = smove.from()
            && let Some(piece) = chess.board().piece_at(from_sq)
            && !smove.is_castle()
        {
            let role = if smove.role() != Role::Pawn {
                piece.char().to_string()
            } else {
                "".to_string()
            };

            let promotion = if let Some(promote_to) = smove.promotion() {
                format!("={}", promote_to.char())
            } else {
                "".to_string()
            };

            let delimiter = if smove.is_capture() { "x" } else { "-" };
            let check_or_mate = san
                .suffix
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default();
            Ok(format!(
                "{role}{from_sq}{delimiter}{}{promotion}{check_or_mate}",
                smove.to()
            ))
        } else {
            assert!(smove.is_put(), "this is a bug in the lib, please report");
            Ok(san_str)
        }
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

            let san = SanPlus::from_move(chess.clone(), smove).to_string();

            if !out.is_empty() {
                out.push(' ');
            }
            if out.is_empty() && !white_to_move {
                // TODO FIXME UPDATE IF python-chess behavior changes
                out.push_str(&format!("{move_number}...{san}"));
            } else if white_to_move {
                out.push_str(&format!("{move_number}. {san}"));
            } else {
                out.push_str(&san);
            }

            chess.play_unchecked(smove);
            if !white_to_move {
                move_number += 1;
            }
            white_to_move = !white_to_move;
        }

        Ok(out)
    }

    // name parse_san for o3
    #[pyo3(name = "parse_san")]
    fn py_parse_san(slf: &Bound<'_, Self>, san: &str) -> PyResult<PyMove> {
        let chess = Self::try_shakmaty(slf)?;
        let m = Self::parse_san(&chess, san)?;
        // println!("san: {san}, move: {m:?}");
        Ok(m.map(Into::into).unwrap_or(PyMove::NULL))
    }

    fn push_san(slf: &Bound<'_, Self>, san: &str) -> PyResult<PyMove> {
        let chess = Self::try_shakmaty(slf)?;
        let m_opt = Self::parse_san(&chess, san)?;
        Self::push(slf, chess, m_opt)?;
        Ok(m_opt.map(Into::into).unwrap_or(PyMove::NULL))
    }

    fn parse_xboard(slf: &Bound<'_, Self>, xboard: &str) -> PyResult<PyMove> {
        Self::py_parse_san(slf, xboard)
    }

    fn push_xboard(slf: &Bound<'_, Self>, xboard: &str) -> PyResult<PyMove> {
        Self::push_san(slf, xboard)
    }

    #[pyo3(signature = (move_obj, chess960=None))]
    fn uci(slf: &Bound<'_, Self>, move_obj: PyMove, chess960: Option<bool>) -> PyResult<String> {
        let chess = Self::try_shakmaty(slf)?;
        let smove = move_obj
            .inner
            .to_move(&chess)
            .map_err(|_| IllegalMoveError::new_err("illegal move"))?;
        let mode = if chess960.unwrap_or(slf.borrow().chess960) {
            shakmaty::CastlingMode::Chess960
        } else {
            shakmaty::CastlingMode::Standard
        };
        Ok(smove.to_uci(mode).to_string())
    }

    fn xboard(slf: &Bound<'_, Self>, move_obj: PyMove) -> PyResult<String> {
        Self::uci(slf, move_obj, None)
    }

    #[pyo3(signature = (chess960, from_square, to_square, promotion=None, drop=None))]
    fn _from_chess960(
        slf: &Bound<'_, Self>,
        chess960: bool,
        from_square: PySquare,
        to_square: PySquare,
        promotion: Option<PyRole>,
        drop: Option<PyRole>,
    ) -> PyResult<PyMove> {
        if !chess960 && promotion.is_none() && drop.is_none() {
            let kings = slf.as_super().borrow().by_role[Role::King];

            if from_square.0 == Square::E1 && kings.contains(Square::E1) {
                if to_square.0 == Square::H1 {
                    return Ok(PyMove {
                        inner: UciMove::Normal {
                            from: Square::E1,
                            to: Square::G1,
                            promotion: None,
                        },
                    });
                }
                if to_square.0 == Square::A1 {
                    return Ok(PyMove {
                        inner: UciMove::Normal {
                            from: Square::E1,
                            to: Square::C1,
                            promotion: None,
                        },
                    });
                }
            } else if from_square.0 == Square::E8 && kings.contains(Square::E8) {
                if to_square.0 == Square::H8 {
                    return Ok(PyMove {
                        inner: UciMove::Normal {
                            from: Square::E8,
                            to: Square::G8,
                            promotion: None,
                        },
                    });
                }
                if to_square.0 == Square::A8 {
                    return Ok(PyMove {
                        inner: UciMove::Normal {
                            from: Square::E8,
                            to: Square::C8,
                            promotion: None,
                        },
                    });
                }
            }
        }

        PyMove::py_new(from_square, to_square, promotion, drop)
    }

    fn is_capture(slf: &Bound<'_, Self>, move_obj: PyMove) -> PyResult<bool> {
        let chess = Self::try_shakmaty(slf)?;
        let smove = move_obj
            .inner
            .to_move(&chess)
            .map_err(|_| IllegalMoveError::new_err("illegal move"))?;
        Ok(smove.is_capture())
    }

    fn is_en_passant(slf: &Bound<'_, Self>, move_obj: PyMove) -> PyResult<bool> {
        let chess = Self::try_shakmaty(slf)?;
        move_obj
            .to_move_unless_null(&chess)
            .map(|m_opt| m_opt.map(|m| m.is_en_passant()).unwrap_or_default())
    }

    fn is_castling(slf: &Bound<'_, Self>, move_obj: PyMove) -> PyResult<bool> {
        let chess = Self::try_shakmaty(slf)?;
        move_obj
            .to_move_unless_null(&chess)
            .map(|m_opt| m_opt.map(|m| m.is_castle()).unwrap_or_default())
    }

    fn is_irreversible(slf: &Bound<'_, Self>, move_obj: PyMove) -> PyResult<bool> {
        let chess = Self::try_shakmaty(slf)?;
        move_obj
            .to_move_unless_null(&chess)
            .map(|m_opt| m_opt.map(|m| chess.is_irreversible(m)).unwrap_or_default())
    }

    #[pyo3(signature = (from_square, to_square, promotion=None))]
    fn find_move(
        slf: &Bound<'_, Self>,
        from_square: PySquare,
        to_square: PySquare,
        promotion: Option<PyRole>,
    ) -> PyResult<PyMove> {
        let chess = Self::try_shakmaty(slf)?;
        let wanted_promotion = promotion.or_else(|| {
            let pawns = slf.as_super().borrow().by_role[Role::Pawn];
            let backrank = Bitboard::BACKRANKS & to_square.0;
            (pawns.contains(from_square.0) && backrank.any()).then_some(PyRole(Role::Queen))
        });
        let board = slf.borrow();
        let move_obj =
            Self::_from_chess960(slf, board.chess960, from_square, to_square, wanted_promotion, None)?;

        for m in chess.legal_moves() {
            if PyMove::from(&m) == move_obj {
                return Ok(move_obj);
            }
        }

        Err(IllegalMoveError::new_err(format!(
            "no matching legal move for {:?} ({:?} -> {:?}) in {}",
            move_obj.inner,
            from_square.0,
            to_square.0,
            Self::fen(slf, false, "legal", None)?,
        )))
    }

    fn has_insufficient_material(slf: &Bound<'_, Self>, color: PyColor) -> PyResult<bool> {
        let chess = Self::try_shakmaty(slf)?;
        Ok(chess.has_insufficient_material(color.0))
    }

    fn has_chess960_castling_rights(slf: &Bound<'_, Self>) -> PyResult<bool> {
        let rights =
            Self::clean_castling_rights_with_960(slf, CastlingMode::Chess960)?.castling_rights();

        // # If there are any castling rights in standard chess, the king must be
        // # on e1 or e8.
        // if castling_rights & BB_RANK_1 and not self.occupied_co[WHITE] & self.kings & BB_E1:
        //     return True
        // if castling_rights & BB_RANK_8 and not self.occupied_co[BLACK] & self.kings & BB_E8:
        //     return True
        if rights.intersects(!Bitboard::CORNERS) {
            return Ok(true);
        }
        if let Some(white_king) = slf.as_super().borrow().king(Color::White)
            && white_king != Square::E1
        {
            return Ok(true);
        }
        if let Some(black_king) = slf.as_super().borrow().king(Color::Black)
            && black_king != Square::E8
        {
            return Ok(true);
        }
        Ok(false)
    }

    fn has_castling_rights(slf: &Bound<'_, Self>, color: PyColor) -> PyResult<bool> {
        Self::clean_castling_rights(slf).map(|c| c.any())
    }

    fn has_kingside_castling_rights(slf: &Bound<'_, Self>, color: PyColor) -> PyResult<bool> {
        Self::clean_castling_rights(slf).map(|c| c.has(color.0, CastlingSide::KingSide))
    }

    #[pyo3(name = "clean_castling_rights")]
    fn py_clean_castling_rights(slf: &Bound<'_, Self>) -> PyResult<u64> {
        Self::clean_castling_rights(slf).map(|c| c.castling_rights().0)
    }

    fn has_queenside_castling_rights(slf: &Bound<'_, Self>, color: PyColor) -> PyResult<bool> {
        Self::clean_castling_rights(slf).map(|c| c.has(color.0, CastlingSide::QueenSide))
    }

    fn status(slf: &Bound<'_, Self>) -> PyResult<u32> {
        let setup = Self::try_setup(slf)?;
        let mode = if slf.borrow().chess960 {
            shakmaty::CastlingMode::Chess960
        } else {
            shakmaty::CastlingMode::Standard
        };

        let status = crate::board_status::status(setup, mode);

        Ok(status.bits())
    }

    fn is_valid(slf: &Bound<'_, Self>) -> PyResult<bool> {
        Ok(Self::try_shakmaty(slf).is_ok())
    }

    fn is_fifty_moves(slf: &Bound<'_, Self>) -> PyResult<bool> {
        let chess = Self::try_shakmaty(slf)?;
        Ok(chess.halfmoves() >= 100 && chess.outcome().is_unknown())
    }

    fn is_seventyfive_moves(slf: &Bound<'_, Self>) -> PyResult<bool> {
        let chess = Self::try_shakmaty(slf)?;
        Ok(chess.halfmoves() >= 150 && chess.outcome().is_unknown())
    }

    // is only about current position
    #[pyo3(signature = (count=3))]
    fn is_repetition(slf: &Bound<'_, Self>, count: usize) -> PyResult<bool> {
        if count <= 1 {
            return Ok(true);
        }

        let board = slf.borrow();
        let base_board = slf.as_super().borrow();
        let key = StateBoard::from((&*board, &*base_board)).transposition_key();
        let mut seen = 1;

        // heuristic, last position come first
        for stack in board._stack.iter().rev() {
            if key == stack.transposition_key() {
                seen += 1;
                if seen == count {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    #[pyo3(name = "is_insufficient_material")]
    fn py_is_insufficient_material(slf: &Bound<'_, Self>) -> PyResult<bool> {
        Ok(Self::try_shakmaty(slf)?.is_insufficient_material())
    }

    fn is_stalemate(slf: &Bound<'_, Self>) -> PyResult<bool> {
        Ok(Self::try_shakmaty(slf)?.is_stalemate())
    }

    fn can_claim_fifty_moves(slf: &Bound<'_, Self>) -> PyResult<bool> {
        let chess = Self::try_shakmaty(slf)?;
        let moves = chess.legal_moves();
        // slight p
        if chess.halfmoves() >= 100 && !moves.is_empty() {
            return Ok(true);
        }
        if chess.halfmoves() == 99 {
            for m in moves.iter() {
                if !m.is_zeroing() {
                    let mut after = chess.clone();
                    after.play_unchecked(*m);
                    if after.outcome().is_unknown() {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    fn is_fivefold_repetition(slf: &Bound<'_, Self>) -> PyResult<bool> {
        Self::is_repetition(slf, 5)
    }

    fn can_claim_threefold_repetition(slf: &Bound<'_, Self>) -> PyResult<bool> {
        let board = slf.borrow();
        let base_board = slf.as_super().borrow();
        let current_key = StateBoard::from((&*board, &*base_board)).transposition_key();

        let mut transpositions: HashMap<TranspositionKey, usize> =
            HashMap::with_capacity(board._stack.len() * 2);
        for stack in board._stack.iter().rev() {
            *transpositions
                .entry(stack.transposition_key().clone())
                .or_insert(0) += 1;
        }

        if *transpositions.get(&current_key).unwrap_or(&0) >= 3 {
            return Ok(true);
        }
        let chess = Self::try_shakmaty(slf)?;
        for m in chess.legal_moves() {
            let mut next_chess = chess.clone();
            next_chess.play_unchecked(m);
            let next_key = Self::get_transposition_key(&next_chess);

            if *transpositions.get(&next_key).unwrap_or(&0) >= 2 {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn can_claim_draw(slf: &Bound<'_, Self>) -> PyResult<bool> {
        Ok(Self::can_claim_fifty_moves(slf)? || Self::can_claim_threefold_repetition(slf)?)
    }

    #[pyo3(signature = (*, claim_draw=false))]
    fn result(slf: &Bound<'_, Self>, claim_draw: bool) -> PyResult<String> {
        let chess = Self::try_shakmaty(slf)?;

        let mut outcome = chess.outcome();

        if outcome == shakmaty::Outcome::Unknown {
            if Self::is_seventyfive_moves(slf)? || Self::is_fivefold_repetition(slf)? {
                outcome = shakmaty::Outcome::Known(shakmaty::KnownOutcome::Draw);
            } else if claim_draw
                && (Self::can_claim_fifty_moves(slf)? || Self::can_claim_threefold_repetition(slf)?)
            {
                outcome = shakmaty::Outcome::Known(shakmaty::KnownOutcome::Draw);
            }
        }

        Ok(outcome.to_string())
    }

    #[pyo3(signature = (epd))]
    fn set_epd(slf: &Bound<'_, Self>, epd: &str) -> PyResult<Py<PyAny>> {
        let py = slf.py();
        let parts = crate::epd_ops::split_epd_fields(epd);

        if parts.len() < 4 {
            return Err(PyValueError::new_err(
                "invalid epd: expected at least 4 fields",
            ));
        }

        if parts.len() > 4 {
            let parse_fen = format!("{} {} {} {} 0 1", parts[0], parts[1], parts[2], parts[3]);
            let (parse_board, parse_base) =
                Self::__new__(py, Some(&parse_fen), slf.borrow().chess960)?;
            let parse_board = Bound::new(py, (parse_board, parse_base))?;
            let operations = crate::epd_ops::parse_epd_ops(&parse_board, parts[4])?;
            let hmvc = crate::epd_ops::hmvc(&operations)?;
            let fmvn = crate::epd_ops::fmvn(&operations)?;

            let fen = format!(
                "{} {} {} {} {} {}",
                parts[0], parts[1], parts[2], parts[3], hmvc, fmvn
            );
            Self::set_fen(slf.borrow_mut(), &fen)?;
            Ok(crate::epd_ops::epd_operations_to_pydict(py, &operations)?.into_any())
        } else {
            let fen = format!("{} {} {} {} 0 1", parts[0], parts[1], parts[2], parts[3]);
            Self::set_fen(slf.borrow_mut(), &fen)?;
            Ok(PyDict::new(py).into_any().unbind())
        }
    }

    #[classmethod]
    #[pyo3(signature = (epd, *, chess960=false))]
    fn from_epd(
        _cls: &Bound<'_, PyType>,
        py: Python<'_>,
        epd: &str,
        chess960: bool,
    ) -> PyResult<Py<PyAny>> {
        let parts = crate::epd_ops::split_epd_fields(epd);
        if parts.len() < 4 {
            return Err(PyValueError::new_err(
                "invalid epd: expected at least 4 fields",
            ));
        }

        let fen = format!("{} {} {} {} 0 1", parts[0], parts[1], parts[2], parts[3]);

        let (board, base) = Self::__new__(py, Some(&fen), chess960)?;
        let board = Bound::new(py, (board, base))?;

        let operations = if parts.len() > 4 {
            let operations = crate::epd_ops::parse_epd_ops(&board, parts[4])?;
            board.borrow_mut().halfmove_clock = crate::epd_ops::hmvc(&operations)? as u16;
            board.borrow_mut().fullmove_number =
                NonZeroU32::new(crate::epd_ops::fmvn(&operations)?)
                    .ok_or_else(|| PyValueError::new_err("invalid fmvn value: 0"))?;

            crate::epd_ops::epd_operations_to_pydict(py, &operations)?.into_any()
        } else {
            PyDict::new(py).into_any().unbind()
        };

        let tuple = PyTuple::new(
            py,
            [board.into_any(), operations.bind(py).clone().into_any()],
        )?;
        Ok(tuple.into_any().unbind())
    }

    fn is_checkmate(slf: &Bound<'_, Self>) -> PyResult<bool> {
        Ok(Self::try_shakmaty(slf)?.is_checkmate())
    }

    fn clear_board(mut slf: PyRefMut<'_, Self>) {
        slf.clear_stack();
        slf.into_super().clear_board();
    }

    #[pyo3(name = "push")]
    fn py_push(slf: &Bound<'_, Self>, move_obj: PyMove) -> PyResult<()> {
        let chess = Self::try_shakmaty(slf)?;

        let m_opt = match move_obj.inner {
            UciMove::Null => None,
            _ => Some(move_obj.inner.to_move(&chess).map_err(|_| {
                IllegalMoveError::new_err(format!(
                    "illegal move {move_obj:?} in position {}",
                    Fen::from_position(&chess, shakmaty::EnPassantMode::Always)
                ))
            })?),
        };
        Self::push(slf, chess, m_opt)
    }

    #[pyo3(name = "parse_uci")]
    fn py_parse_uci(slf: &Bound<'_, Self>, uci: &str) -> PyResult<PyMove> {
        let chess = Self::try_shakmaty(slf)?;
        Ok(Self::parse_uci(&chess, uci)?
            .map(Into::into)
            .unwrap_or(PyMove::NULL))
    }

    fn push_uci(slf: &Bound<'_, Self>, uci: &str) -> PyResult<PyMove> {
        let chess = Self::try_shakmaty(slf)?;
        let m_opt = Self::parse_uci(&chess, uci)?;
        Self::push(slf, chess, m_opt)?;

        Ok(m_opt.map(Into::into).unwrap_or(PyMove::NULL))
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
            .is_ok_and(|m| chess.is_legal(m)))
    }

    #[pyo3(signature = (move_obj))]
    fn is_pseudo_legal(slf: &Bound<'_, Self>, move_obj: PyMove) -> PyResult<bool> {
        let moves = Self::generate_pseudo_legal_moves(slf, Bitboard::FULL.0, Bitboard::FULL.0)?;
        Ok(moves.contains(&move_obj))
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
            Self::mut_from_chess_but_stack(&root, &chess);
        }

        Ok(root)
    }

    fn mirror(slf: &Bound<'_, Self>) -> PyResult<Py<PyAny>> {
        let py_board = slf.call_method0("copy")?;
        py_board.call_method0("apply_mirror")?;
        Ok(py_board.into_any().unbind())
    }

    #[pyo3(signature = (*, claim_draw=None))]
    fn is_game_over(slf: &Bound<'_, Self>, claim_draw: Option<bool>) -> PyResult<bool> {
        let chess = Self::try_shakmaty(slf)?;
        if chess.is_game_over() {
            return Ok(true);
        }

        if Self::is_seventyfive_moves(slf)? {
            return Ok(true);
        }

        if Self::is_fivefold_repetition(slf)? {
            return Ok(true);
        }

        if claim_draw.unwrap_or(false) {
            if Self::can_claim_fifty_moves(slf)? {
                return Ok(true);
            }
            if Self::can_claim_threefold_repetition(slf)? {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

impl Board {
    fn clean_castling_rights(slf: &Bound<'_, Self>) -> PyResult<Castles> {
        let board = slf.borrow();
        let mode = if board.chess960 {
            CastlingMode::Chess960
        } else {
            shakmaty::CastlingMode::Standard
        };

        Self::clean_castling_rights_with_960(slf, mode)
    }

    fn clean_castling_rights_with_960(
        slf: &Bound<'_, Self>,
        mode: CastlingMode,
    ) -> PyResult<Castles> {
        let setup = Self::try_setup(slf)?;

        Ok(Castles::from_setup(&setup, mode).unwrap_or_else(|c| c))
    }

    fn get_transposition_key(chess: &shakmaty::Chess) -> TranspositionKey {
        let (by_role, by_color) = chess.board().clone().into_bitboards();
        (
            by_role,
            by_color,
            chess.promoted(),
            chess.turn(),
            chess.castles().castling_rights(),
            chess.ep_square(shakmaty::EnPassantMode::Legal),
        )
    }

    fn parse_san(chess: &Chess, san: &str) -> PyResult<Option<Move>> {
        // python-chess parser is very lenient and accepts uci as san, so we try to parse as uci first to avoid that
        let uci_parsed = UciMove::from_str(san);
        if let Ok(uci_move) = uci_parsed {
            if !matches!(uci_move, UciMove::Null) {
                // check if legal
                return Ok(Some(uci_move.to_move(chess).map_err(|_| {
                    IllegalMoveError::new_err(format!("illegal san as valid uci: {san:?}"))
                })?));
            } else {
                return Ok(None);
            }
        }
        let parsed = San::from_str(san)
            .map_err(|_| InvalidMoveError::new_err(format!("invalid san: {san:?}")))?;

        if matches!(parsed, San::Null) {
            return Ok(None);
        }

        Ok(Some(parsed.to_move(chess).map_err(|e| match e {
            SanError::IllegalSan => IllegalMoveError::new_err(format!("illegal san move: {san}")),
            SanError::AmbiguousSan => {
                AmbiguousMoveError::new_err(format!("ambiguous san move: {san}"))
            }
        })?))
    }

    fn parse_uci(chess: &Chess, uci: &str) -> PyResult<Option<Move>> {
        let inner = UciMove::from_str(uci)
            .map_err(|_| InvalidMoveError::new_err(format!("invalid uci: {uci:?}")))?;

        if !matches!(inner, UciMove::Null) {
            // check if legal
            return Ok(Some(inner.to_move(chess).map_err(|_| {
                IllegalMoveError::new_err(format!("illegal uci: {uci:?}"))
            })?));
        }
        Ok(None)
    }

    // m None is NullMove
    fn push(slf: &Bound<'_, Self>, chess: Chess, m_opt: Option<Move>) -> PyResult<()> {
        let board_state = {
            let rust_board = slf.borrow();
            let base_board = slf.as_super().borrow();
            StateBoard::from((&*rust_board, &*base_board))
        };
        if let Some(m) = m_opt {
            let new_chess = chess
                .play(m)
                .map_err(|e| IllegalMoveError::new_err(format!("illegal move: {e}")))?;
            Self::mut_from_chess_but_stack(slf, &new_chess);
        } else {
            // null move, just update turn and ep_square
            let mut rust_board = slf.borrow_mut();
            rust_board.turn = chess.turn().other();
            rust_board.ep_square = None;
            rust_board.halfmove_clock += 1;
            // we've already swapped color at that point
            if rust_board.turn == Color::White {
                rust_board.fullmove_number = rust_board.fullmove_number.saturating_add(1);
            }
        }

        let mut rust_board = slf.borrow_mut();
        rust_board
            .move_stack
            .push(m_opt.map(Into::into).unwrap_or(PyMove::NULL));
        rust_board._stack.push(board_state);

        Ok(())
    }

    fn mut_from_chess_but_stack(slf: &Bound<'_, Self>, chess: &Chess) {
        {
            let mut rust_board = slf.borrow_mut();
            rust_board.turn = chess.turn();
            rust_board.castling_rights = chess.castles().castling_rights();
            rust_board.ep_square = chess.ep_square(shakmaty::EnPassantMode::Always);
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

    fn mut_from_setup_but_stack_fullmove(slf: &Bound<'_, Self>, setup: &Setup) {
        {
            let mut rust_board = slf.borrow_mut();
            rust_board.turn = setup.turn;
            rust_board.castling_rights = setup.castling_rights;
            rust_board.ep_square = setup.ep_square;
            rust_board.halfmove_clock = setup.halfmoves as u16;
        }

        let (roles, colors) = setup.board.clone().into_bitboards();
        let promoted = setup.promoted;

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
        base.by_role = state.by_role;
        base.by_color = state.by_color;
        base.promoted = state.promoted;
    }

    fn try_shakmaty_with_promoted(
        slf: &Bound<'_, Self>,
        include_promoted: bool,
    ) -> PyResult<Chess> {
        let is_960 = slf.borrow().chess960;
        Chess::from_setup(
            Self::try_setup_with_promoted(slf, include_promoted)?,
            if is_960 {
                CastlingMode::Chess960
            } else {
                CastlingMode::Standard
            },
        )
        .or_else(shakmaty::PositionError::ignore_too_much_material)
        .or_else(shakmaty::PositionError::ignore_impossible_check)
        .or_else(shakmaty::PositionError::ignore_invalid_castling_rights)
        .or_else(shakmaty::PositionError::ignore_invalid_ep_square)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid state: {e:?}")))
    }

    // &Bound<'_, Self> to be able to acess BaseBoard
    fn try_shakmaty(slf: &Bound<'_, Self>) -> PyResult<Chess> {
        Self::try_shakmaty_with_promoted(slf, true)
    }

    // &Bound<'_, Self> to be able to acess BaseBoard
    fn try_setup(slf: &Bound<'_, Self>) -> PyResult<Setup> {
        Self::try_setup_with_promoted(slf, true)
    }

    fn try_setup_with_promoted(slf: &Bound<'_, Self>, include_promoted: bool) -> PyResult<Setup> {
        let board = slf.borrow();
        let base_board = slf.as_super().borrow();

        let b = base_board.board()?;

        Ok(Setup {
            board: b,
            promoted: if include_promoted {
                base_board.promoted
            } else {
                Bitboard::EMPTY
            },
            pockets: None,
            turn: board.turn,
            castling_rights: board.castling_rights,
            ep_square: board.ep_square,
            remaining_checks: None,
            halfmoves: u32::from(board.halfmove_clock),
            fullmoves: board.fullmove_number,
        })
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

    // Private helper for move generation
    fn generate_x_moves_legal_or_pseudo_impl<F, G>(
        slf: &Bound<'_, Self>,
        pseudo_or_legal: G,
        from_mask: u64,
        to_mask: u64,
        mut filter: F,
    ) -> PyResult<Vec<PyMove>>
    where
        F: FnMut(&shakmaty::Move) -> bool,
        G: Fn(&Chess) -> MoveList,
    {
        let chess = Self::try_shakmaty(slf)?;
        let from = Bitboard(from_mask);
        let to = Bitboard(to_mask);
        let mut moves = pseudo_or_legal(&chess);
        moves.retain(|m| {
            m.from().is_none_or(|sq| from.contains(sq)) && to.contains(m.to()) && filter(m)
        });
        Ok(moves.into_iter().map(Into::into).collect())
    }

    // Private helper for move generation
    fn gen_pseudo_moves_and_filter<F>(
        slf: &Bound<'_, Self>,
        from_mask: u64,
        to_mask: u64,
        filter: F,
    ) -> PyResult<Vec<PyMove>>
    where
        F: FnMut(&shakmaty::Move) -> bool,
    {
        Self::generate_x_moves_legal_or_pseudo_impl(
            slf,
            |x| x.pseudo_legal_moves().0,
            from_mask,
            to_mask,
            filter,
        )
    }

    // Private helper for move generation
    fn gen_legal_moves_and_filter<F>(
        slf: &Bound<'_, Self>,
        from_mask: u64,
        to_mask: u64,
        filter: F,
    ) -> PyResult<Vec<PyMove>>
    where
        F: FnMut(&shakmaty::Move) -> bool,
    {
        Self::generate_x_moves_legal_or_pseudo_impl(
            slf,
            Chess::legal_moves,
            from_mask,
            to_mask,
            filter,
        )
    }
}
#[pyclass(module = "rust_chess", name = "PseudoLegalMoveGeneratorIter")]
pub struct PseudoLegalMoveGeneratorIter {
    moves: std::vec::IntoIter<PyMove>,
}

#[pymethods]
impl PseudoLegalMoveGeneratorIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyMove> {
        slf.moves.next()
    }
}

#[pyclass(module = "rust_chess", name = "PseudoLegalMoveGenerator")]
pub struct PseudoLegalMoveGenerator {
    board: Py<Board>,
}

#[pymethods]
impl PseudoLegalMoveGenerator {
    #[new]
    fn py_new(board: Py<Board>) -> Self {
        Self { board }
    }

    fn __bool__(&self, py: Python<'_>) -> PyResult<bool> {
        let board = self.board.bind(py);
        let moves = Board::generate_pseudo_legal_moves(board, Bitboard::FULL.0, Bitboard::FULL.0)?;
        Ok(!moves.is_empty())
    }

    fn __len__(&self, py: Python<'_>) -> PyResult<usize> {
        self.count(py)
    }

    fn count(&self, py: Python<'_>) -> PyResult<usize> {
        let board = self.board.bind(py);
        let moves = Board::generate_pseudo_legal_moves(board, Bitboard::FULL.0, Bitboard::FULL.0)?;
        Ok(moves.len())
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<PseudoLegalMoveGeneratorIter> {
        let board = self.board.bind(py);
        let moves = Board::generate_pseudo_legal_moves(board, Bitboard::FULL.0, Bitboard::FULL.0)?;
        Ok(PseudoLegalMoveGeneratorIter {
            moves: moves.into_iter(),
        })
    }

    fn __contains__(&self, move_obj: PyMove, py: Python<'_>) -> PyResult<bool> {
        let board = self.board.bind(py);
        Board::is_pseudo_legal(board, move_obj)
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        let py = slf.py();
        let self_rust = slf.borrow();
        let board = self_rust.board.bind(py);
        let chess = Board::try_shakmaty(board)?;

        let moves = chess.pseudo_legal_moves().0;
        // inefficient but this is debug code...
        let mut sans = Vec::new();
        moves.into_iter().for_each(|m| {
            let c = chess.clone();
            sans.push(if c.is_legal(m) {
                SanPlus::from_move(c, m).to_string()
            } else {
                UciMove::from_move(m, shakmaty::CastlingMode::Chess960).to_string()
            });
        });

        Ok(format!(
            "<PseudoLegalMoveGenerator at {:#x} ({})>",
            slf.as_ptr() as usize,
            sans.join(", ")
        ))
    }
}

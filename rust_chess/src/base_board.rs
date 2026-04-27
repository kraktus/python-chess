use pyo3::exceptions::{PyIndexError, PyNotImplementedError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyType};
use shakmaty::{Bitboard, Board, Color, Piece, Role, Square};
use std::str::FromStr;

use crate::piece::PyPiece;
use crate::square_set::SquareSet;

use crate::util::{PyColor, PyRole, PySquare};

#[pyclass(module = "rust_chess", name = "OccupiedCo")]
pub struct OccupiedCo {
    board: Py<BaseBoard>,
}

#[pymethods]
impl OccupiedCo {
    fn __getitem__(&self, index: usize, py: Python) -> PyResult<u64> {
        let board = self.board.bind(py).borrow();
        if index == 1 {
            Ok(board.board.by_color(Color::White).0)
        } else if index == 0 {
            Ok(board.board.by_color(Color::Black).0)
        } else {
            Err(PyIndexError::new_err("Index out of bounds"))
        }
    }

    fn __setitem__(&mut self, index: usize, value: u64, py: Python) -> PyResult<()> {
        let mut board = self.board.bind(py).borrow_mut();
        if index == 1 {
            board.set_occupied_w(value);
        } else if index == 0 {
            board.set_occupied_b(value);
        } else {
            return Err(PyIndexError::new_err("Index out of bounds"));
        }
        Ok(())
    }

    fn __repr__(&self, py: Python) -> String {
        let board = self.board.bind(py).borrow();
        format!(
            "[{}, {}]",
            board.board.by_color(Color::Black).0,
            board.board.by_color(Color::White).0
        )
    }
}

#[pyclass(subclass, dict, module = "rust_chess", name = "BaseBoard")]
#[derive(Clone, Default, PartialEq, Eq)]
pub struct BaseBoard {
    pub board: shakmaty::Board,
    pub promoted: shakmaty::Bitboard,
}

#[pymethods]
impl BaseBoard {
    #[new]
    #[pyo3(signature = (board_fen=Some("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR")))]
    fn py_new(board_fen: Option<&str>) -> PyResult<Self> {
        if let Some(fen) = board_fen {
            return Ok(if fen == "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR" {
                Self::default()
            } else {
                let mut b = Self::empty();
                b.set_board_fen(fen)?;
                b
            });
        }
        Ok(Self::empty())
    }

    #[getter]
    fn pawns(&self) -> u64 {
        self.board.by_role(Role::Pawn).0
    }
    #[setter]
    fn set_pawns(&mut self, value: u64) {
        self._set_role(Role::Pawn, value);
    }
    #[getter]
    fn knights(&self) -> u64 {
        self.board.by_role(Role::Knight).0
    }
    #[setter]
    fn set_knights(&mut self, value: u64) {
        self._set_role(Role::Knight, value);
    }
    #[getter]
    fn bishops(&self) -> u64 {
        self.board.by_role(Role::Bishop).0
    }
    #[setter]
    fn set_bishops(&mut self, value: u64) {
        self._set_role(Role::Bishop, value);
    }
    #[getter]
    fn rooks(&self) -> u64 {
        self.board.by_role(Role::Rook).0
    }
    #[setter]
    fn set_rooks(&mut self, value: u64) {
        self._set_role(Role::Rook, value);
    }
    #[getter]
    fn queens(&self) -> u64 {
        self.board.by_role(Role::Queen).0
    }
    #[setter]
    fn set_queens(&mut self, value: u64) {
        self._set_role(Role::Queen, value);
    }
    #[getter]
    fn kings(&self) -> u64 {
        self.board.by_role(Role::King).0
    }
    #[setter]
    fn set_kings(&mut self, value: u64) {
        self._set_role(Role::King, value);
    }
    #[getter]
    fn promoted(&self) -> u64 {
        self.promoted.0
    }
    #[getter]
    fn occupied(&self) -> u64 {
        self.board.occupied().0
    }
    #[setter]
    fn set_occupied(&mut self, _value: u64) { /* Ignore */
    }
    #[setter]
    fn set_promoted(&mut self, value: u64) {
        self.promoted = Bitboard(value);
    }

    #[getter]
    fn occupied_co(slf: Py<Self>, py: Python) -> PyResult<OccupiedCo> {
        Ok(OccupiedCo {
            board: slf.clone_ref(py),
        })
    }

    #[setter]
    fn set_occupied_co(slf: Py<Self>, py: Python, value: Vec<u64>) -> PyResult<()> {
        if value.len() != 2 {
            return Err(PyValueError::new_err("occupied_co must be length 2"));
        }
        let mut board = slf.bind(py).borrow_mut();
        board.set_occupied_b(value[0]);
        board.set_occupied_w(value[1]);
        Ok(())
    }

    fn clear_board(&mut self) {
        self.board = Board::empty();
        self.promoted = Bitboard(0);
    }

    fn reset_board(&mut self) {
        self.board = Board::new();
        self.promoted = Bitboard(0);
    }

    fn set_board_fen(&mut self, fen: &str) -> PyResult<()> {
        self.board =
            Board::from_str(fen).map_err(|e| PyValueError::new_err(format!("invalid fen: {e}")))?;
        self.promoted = Bitboard(0);
        Ok(())
    }

    fn piece_count(&self) -> u32 {
        self.board.occupied().count() as u32
    }

    fn pieces_mask(&self, piece_type: PyRole, color: PyColor) -> u64 {
        self.board
            .by_piece(Piece {
                color: color.0,
                role: piece_type.0,
            })
            .0
    }

    fn pieces(&self, piece_type: PyRole, color: PyColor) -> SquareSet {
        SquareSet {
            bb: Bitboard(self.pieces_mask(piece_type, color)),
        }
    }

    fn piece_type_at(&self, square: PySquare) -> Option<u8> {
        self.board.role_at(square.0).map(|r| r as u8)
    }

    fn color_at(&self, square: PySquare) -> Option<bool> {
        self.board.color_at(square.0).map(|c| c.is_white())
    }

    fn piece_at(&self, square: PySquare) -> Option<PyPiece> {
        self.board.piece_at(square.0).map(PyPiece)
    }

    fn king(&self, color: PyColor) -> Option<u8> {
        (self.board.by_role(Role::King) & self.board.by_color(color.0) & !self.promoted)
            .single_square()
            .map(|sq| sq as u8)
    }

    fn attacks_mask(&self, square: PySquare) -> u64 {
        self.board.attacks_from(square.0).0
    }
    fn attacks(&self, square: PySquare) -> SquareSet {
        SquareSet {
            bb: Bitboard(self.attacks_mask(square)),
        }
    }

    // TODO FIXME, move to shakmaty
    #[pyo3(signature = (color, square, occupied=None))]
    fn attackers_mask(&self, color: PyColor, square: PySquare, occupied: Option<u64>) -> u64 {
        let occ = occupied.map(Bitboard).unwrap_or(self.board.occupied());
        self.board.attacks_to(square.0, color.0, occ).0
    }
    fn is_attacked_by(
        &self,
        color: PyColor,
        square: PySquare,
        occupied: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<bool> {
        let occ = if let Some(py_occ) = occupied {
            if let Ok(mask) = py_occ.extract::<u64>() {
                Some(mask)
            } else if let Ok(ss) = py_occ.extract::<PyRef<'_, SquareSet>>() {
                Some(ss.bb.0)
            } else {
                None
            }
        } else {
            None
        };
        Ok(self.attackers_mask(color, square, occ) != 0)
    }

    #[pyo3(signature = (color, square, occupied=None))]
    fn attackers(
        &self,
        color: PyColor,
        square: PySquare,
        occupied: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<SquareSet> {
        let occ = if let Some(py_occ) = occupied {
            if let Ok(mask) = py_occ.extract::<u64>() {
                Some(mask)
            } else if let Ok(ss) = py_occ.extract::<PyRef<'_, SquareSet>>() {
                Some(ss.bb.0)
            } else {
                None
            }
        } else {
            None
        };
        Ok(SquareSet {
            bb: Bitboard(self.attackers_mask(color, square, occ)),
        })
    }

    // TODO FIXME, move to shakmaty
    fn pin_mask(&self, color: PyColor, square: PySquare) -> u64 {
        let king_sq_opt = self.king(crate::util::PyColor(color.0));
        if king_sq_opt.is_none() {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        let king_sq = king_sq_opt.unwrap();
        if king_sq == (square.0 as u8) {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        let k_sq = Square::new(king_sq as u32);

        let c_color = color.0;
        let snipers = (shakmaty::attacks::rook_attacks(k_sq, Bitboard(0))
            & (self.board.by_role(Role::Rook) | self.board.by_role(Role::Queen)))
            | (shakmaty::attacks::bishop_attacks(k_sq, Bitboard(0))
                & (self.board.by_role(Role::Bishop) | self.board.by_role(Role::Queen)));
        let enemy_snipers = snipers & self.board.by_color(!c_color);

        for sniper_sq in enemy_snipers {
            let ray = shakmaty::attacks::ray(k_sq, sniper_sq);
            if ray.contains(square.0) {
                let between = shakmaty::attacks::between(k_sq, sniper_sq);
                if (between & self.board.occupied() & !Bitboard(1 << (square.0 as u8))).is_empty() {
                    return ray.0;
                }
            }
        }
        0xFFFF_FFFF_FFFF_FFFF
    }

    fn pin(&self, color: PyColor, square: PySquare) -> SquareSet {
        SquareSet {
            bb: Bitboard(self.pin_mask(color, square)),
        }
    }

    fn is_pinned(&self, color: PyColor, square: PySquare) -> bool {
        self.pin_mask(color, square) != 0xFFFF_FFFF_FFFF_FFFF
    }

    fn remove_piece_at(&mut self, square: PySquare) -> Option<PyPiece> {
        let piece = self.board.remove_piece_at(square.0)?;
        self.promoted.discard(square.0);
        Some(piece.into())
    }

    #[pyo3(signature = (square, piece, promoted=false))]
    fn set_piece_at(
        &mut self,
        square: PySquare,
        piece: Option<&Bound<'_, PyAny>>,
        promoted: bool,
    ) -> PyResult<()> {
        if let Some(py_piece) = piece {
            let p = py_piece.extract::<PyRef<'_, PyPiece>>()?;
            self.board.discard_piece_at(square.0);
            self.board.set_piece_at(square.0, p.0);
            if promoted {
                self.promoted.add(square.0);
            } else {
                self.promoted.discard(square.0);
            }
        } else {
            self.board.discard_piece_at(square.0);
            self.promoted.discard(square.0);
        }
        Ok(())
    }

    #[pyo3(signature = (promoted=None))]
    fn board_fen(&self, promoted: Option<bool>) -> PyResult<String> {
        self.board
            .board_fen_with_promoted(if promoted.unwrap_or(false) {
                self.promoted
            } else {
                Bitboard(0)
            })
            .map_err(|e| PyValueError::new_err(format!("Couldn't produce FEN, error: {e:?}")))
            .map(|x| x.to_string())
    }

    #[classmethod]
    #[pyo3(name = "empty")]
    fn py_empty(_cls: &Bound<'_, PyType>) -> PyResult<Self> {
        Ok(Self::default())
    }

    fn copy(&self) -> Self {
        self.clone()
    }
    fn __copy__(&self) -> Self {
        self.clone()
    }
    fn __deepcopy__(&self, _memo: &Bound<'_, PyAny>) -> Self {
        self.clone()
    }
    fn __eq__(&self, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        if let Ok(other_board) = other.extract::<PyRef<'_, BaseBoard>>() {
            Ok(self == &*other_board)
        } else {
            Err(PyNotImplementedError::new_err(format!(
                "Cannot compare BaseBoard and {other:?}"
            )))
        }
    }
    fn __repr__(&self) -> String {
        format!(
            "BaseBoard('{}')",
            self.board_fen(None).unwrap_or_else(|e| format!("{e:?}"))
        )
    }

    fn __str__(&self) -> String {
        let mut builder = String::with_capacity(150);
        for square in shakmaty::Square::ALL.into_iter().rev() {
            let mask = 1u64 << (square as u8);
            if (self.board.occupied().0 & mask) == 0 {
                builder.push('.');
            } else {
                let is_white = self.color_at(PySquare(square)).unwrap();
                let mut symbol = match self.piece_type_at(PySquare(square)) {
                    Some(1) => 'p',
                    Some(2) => 'n',
                    Some(3) => 'b',
                    Some(4) => 'r',
                    Some(5) => 'q',
                    Some(6) => 'k',
                    _ => '?',
                };
                if is_white {
                    symbol = symbol.to_ascii_uppercase();
                }
                builder.push(symbol);
            }
            if square.file() == shakmaty::File::H {
                if square != shakmaty::Square::H1 {
                    builder.push('\n');
                }
            } else {
                builder.push(' ');
            }
        }
        builder
    }

    fn apply_transform(&mut self, f: &Bound<'_, PyAny>) -> PyResult<()> {
        let pawns = self.pawns();
        let knights = self.knights();
        let bishops = self.bishops();
        let rooks = self.rooks();
        let queens = self.queens();
        let kings = self.kings();
        let occupied_w = self.occupied_w();
        let occupied_b = self.occupied_b();
        let promoted = self.promoted();

        let apply = |bb: u64| -> PyResult<u64> { f.call1((bb,))?.extract::<u64>() };

        self.set_pawns(apply(pawns)?);
        self.set_knights(apply(knights)?);
        self.set_bishops(apply(bishops)?);
        self.set_rooks(apply(rooks)?);
        self.set_queens(apply(queens)?);
        self.set_kings(apply(kings)?);
        self.set_occupied_w(apply(occupied_w)?);
        self.set_occupied_b(apply(occupied_b)?);
        self.set_promoted(apply(promoted)?);
        Ok(())
    }

    fn transform(slf: &Bound<'_, Self>, f: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let board = slf.call_method0("copy")?;
        board.call_method1("apply_transform", (f,))?;
        Ok(board.unbind())
    }

    fn mirror(&self) -> Self {
        let mut board = self.clone();
        board.board.mirror();
        board.promoted = Bitboard(board.promoted.0.swap_bytes());
        board
    }
}

impl BaseBoard {
    fn empty() -> Self {
        Self {
            board: Board::empty(),
            promoted: Bitboard::EMPTY,
        }
    }

    fn occupied_w(&self) -> u64 {
        self.board.by_color(Color::White).0
    }
    fn set_occupied_w(&mut self, value: u64) {
        self._set_color(Color::White, value);
    }
    fn occupied_b(&self) -> u64 {
        self.board.by_color(Color::Black).0
    }
    fn set_occupied_b(&mut self, value: u64) {
        self._set_color(Color::Black, value);
    }

    pub fn _set_role(&mut self, role: Role, value: u64) {
        let (mut by_role, mut by_color) = self.board.clone().into_bitboards();
        let new_mask = Bitboard(value);
        let current_mask = *by_role.get(role);

        let to_remove = current_mask & !new_mask;
        let to_add = new_mask & !current_mask;

        *by_role.get_mut(role) &= !to_remove;
        by_color.white &= !to_remove;
        by_color.black &= !to_remove;

        by_role.pawn &= !to_add;
        by_role.knight &= !to_add;
        by_role.bishop &= !to_add;
        by_role.rook &= !to_add;
        by_role.queen &= !to_add;
        by_role.king &= !to_add;

        *by_role.get_mut(role) |= to_add;

        let empty_before = !(by_color.white | by_color.black);
        by_color.black |= to_add & empty_before;

        if let Ok(new_board) = Board::try_from_bitboards(by_role, by_color) {
            self.board = new_board;
        }
    }

    pub fn _set_color(&mut self, color: Color, value: u64) {
        let (mut by_role, mut by_color) = self.board.clone().into_bitboards();
        let new_mask = Bitboard(value);
        let current_mask = *by_color.get(color);

        let to_remove = current_mask & !new_mask;
        let to_add = new_mask & !current_mask;

        by_role.pawn &= !to_remove;
        by_role.knight &= !to_remove;
        by_role.bishop &= !to_remove;
        by_role.rook &= !to_remove;
        by_role.queen &= !to_remove;
        by_role.king &= !to_remove;

        *by_color.get_mut(color) &= !to_remove;

        *by_color.get_mut(!color) &= !to_add;
        *by_color.get_mut(color) |= to_add;

        let empty_before = !(by_role.pawn
            | by_role.knight
            | by_role.bishop
            | by_role.rook
            | by_role.queen
            | by_role.king);
        by_role.pawn |= to_add & empty_before;

        if let Ok(new_board) = Board::try_from_bitboards(by_role, by_color) {
            self.board = new_board;
        }
    }
}

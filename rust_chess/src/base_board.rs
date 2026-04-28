use crate::piece::PyPiece;
use crate::square_set::SquareSet;
use crate::util::{IntoSquareSet, PyColor, PyRole, PySquare};
use pyo3::exceptions::{PyIndexError, PyNotImplementedError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyType;
use shakmaty::{Bitboard, Board, Piece, Role, Square};
use std::str::FromStr;

#[pyclass(module = "rust_chess", name = "OccupiedCo")]
pub struct OccupiedCo {
    board: Py<BaseBoard>,
}

#[pymethods]
impl OccupiedCo {
    fn __getitem__(&self, index: usize, py: Python) -> PyResult<u64> {
        let board = self.board.bind(py).borrow();
        if index == 1 {
            Ok(board.by_color.white.0)
        } else if index == 0 {
            Ok(board.by_color.black.0)
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
        format!("[{}, {}]", board.by_color.black.0, board.by_color.white.0)
    }
}

#[pyclass(
    subclass,
    dict,
    module = "rust_chess",
    name = "BaseBoard",
    from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub struct BaseBoard {
    pub by_role: shakmaty::ByRole<Bitboard>,
    pub by_color: shakmaty::ByColor<Bitboard>,
    pub promoted: Bitboard,
}

impl Default for BaseBoard {
    fn default() -> Self {
        let (roles, colors) = Board::default().into_bitboards();
        Self {
            by_role: roles,
            by_color: colors,
            promoted: Bitboard::EMPTY,
        }
    }
}

impl BaseBoard {
    pub fn board(&self) -> PyResult<Board> {
        Board::try_from_bitboards(self.by_role.clone(), self.by_color.clone())
            .map_err(|e| PyValueError::new_err(format!("Invalid board state: {e}")))
    }
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
                let mut b = Self::empty()?;
                b.set_board_fen(fen)?;
                b
            });
        }
        Self::empty()
    }

    #[allow(clippy::new_without_default)]
    #[pyo3(signature = (board_fen=Some("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR")))]
    fn __init__(&mut self, board_fen: Option<&str>) -> PyResult<()> {
        if let Some(fen) = board_fen {
            if fen == "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR" {
                let default = Self::default();
                self.by_role = default.by_role;
                self.by_color = default.by_color;
                self.promoted = default.promoted;
            } else {
                self.set_board_fen(fen)?;
            }
        } else {
            let empty = Self::empty()?;
            self.by_role = empty.by_role;
            self.by_color = empty.by_color;
            self.promoted = empty.promoted;
        }
        Ok(())
    }

    #[getter]
    fn pawns(&self) -> u64 {
        self.by_role.pawn.0
    }
    #[setter]
    fn set_pawns(&mut self, value: u64) {
        self.by_role.pawn = Bitboard(value);
    }
    #[getter]
    fn knights(&self) -> u64 {
        self.by_role.knight.0
    }
    #[setter]
    fn set_knights(&mut self, value: u64) {
        self.by_role.knight = Bitboard(value);
    }
    #[getter]
    fn bishops(&self) -> u64 {
        self.by_role.bishop.0
    }
    #[setter]
    fn set_bishops(&mut self, value: u64) {
        self.by_role.bishop = Bitboard(value);
    }
    #[getter]
    fn rooks(&self) -> u64 {
        self.by_role.rook.0
    }
    #[setter]
    fn set_rooks(&mut self, value: u64) {
        self.by_role.rook = Bitboard(value);
    }
    #[getter]
    fn queens(&self) -> u64 {
        self.by_role.queen.0
    }
    #[setter]
    fn set_queens(&mut self, value: u64) {
        self.by_role.queen = Bitboard(value);
    }
    #[getter]
    fn kings(&self) -> u64 {
        self.by_role.king.0
    }
    #[setter]
    fn set_kings(&mut self, value: u64) {
        self.by_role.king = Bitboard(value);
    }
    #[getter]
    fn promoted(&self) -> u64 {
        self.promoted.0
    }
    #[getter]
    #[pyo3(name = "occupied")]
    fn py_occupied(&self) -> u64 {
        self.occupied().0
    }
    #[setter]
    fn set_occupied(&mut self, _value: u64) {
        /* Ignore */
        // Assume that whoever call occupied will also update each color bitboard
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

    pub fn set_board_fen(&mut self, fen: &str) -> PyResult<()> {
        let board =
            Board::from_str(fen).map_err(|e| PyValueError::new_err(format!("invalid fen: {e}")))?;
        let (roles, colors) = board.into_bitboards();
        self.by_role = roles;
        self.by_color = colors;
        self.promoted = Bitboard(0);
        Ok(())
    }

    fn piece_count(&self) -> u32 {
        (self.by_color.white | self.by_color.black).count() as u32
    }

    fn pieces_mask(&self, piece_type: PyRole, color: PyColor) -> u64 {
        (*self.by_color.get(color.0) & *self.by_role.get(piece_type.0)).0
    }

    fn pieces(&self, piece_type: PyRole, color: PyColor) -> SquareSet {
        SquareSet {
            bb: Bitboard(self.pieces_mask(piece_type, color)),
        }
    }

    fn piece_type_at(&self, square: PySquare) -> Option<u8> {
        self.by_role.find(|r| r.contains(square.0)).map(|r| r as u8)
    }

    fn color_at(&self, square: PySquare) -> Option<bool> {
        self.by_color
            .find(|c| c.contains(square.0))
            .map(|c| c.is_white())
    }

    fn piece_at(&self, square: PySquare) -> Option<PyPiece> {
        self.by_role
            .find(|r| r.contains(square.0))
            .and_then(|role| {
                self.by_color
                    .find(|c| c.contains(square.0))
                    .map(|color| PyPiece(Piece { role, color }))
            })
    }

    fn king(&self, color: PyColor) -> Option<u8> {
        (*self.by_role.get(Role::King) & *self.by_color.get(color.0) & !self.promoted)
            .single_square()
            .map(|sq| sq as u8)
    }

    // TODO? remove from pyclass and make pure rust function? undocumented in python
    #[pyo3(name = "attacks_mask")]
    fn py_attacks_mask(&self, square: PySquare) -> u64 {
        self.attacks_mask(square.0).0
    }

    fn attacks(&self, square: PySquare) -> SquareSet {
        SquareSet {
            bb: self.attacks_mask(square.0),
        }
    }
    #[pyo3(signature = (color, square, occupied=None))]
    fn is_attacked_by(
        &self,
        color: PyColor,
        square: PySquare,
        occupied: Option<IntoSquareSet>,
    ) -> PyResult<bool> {
        Ok(self
            .attackers_mask(color, square, occupied.map(|x| x.0))?
            .any())
    }

    #[pyo3(signature = (color, square, occupied=None))]
    fn attackers(
        &self,
        color: PyColor,
        square: PySquare,
        occupied: Option<IntoSquareSet>,
    ) -> PyResult<SquareSet> {
        Ok(SquareSet {
            bb: self.attackers_mask(color, square, occupied.map(|x| x.0))?,
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
            & (*self.by_role.get(Role::Rook) | *self.by_role.get(Role::Queen)))
            | (shakmaty::attacks::bishop_attacks(k_sq, Bitboard(0))
                & (*self.by_role.get(Role::Bishop) | *self.by_role.get(Role::Queen)));
        let enemy_snipers = snipers & *self.by_color.get(!c_color);

        for sniper_sq in enemy_snipers {
            let ray = shakmaty::attacks::ray(k_sq, sniper_sq);
            if ray.contains(square.0) {
                let between = shakmaty::attacks::between(k_sq, sniper_sq);
                if (between
                    & (self.by_color.white | self.by_color.black)
                    & !Bitboard(1 << (square.0 as u8)))
                .is_empty()
                {
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

    pub fn remove_piece_at(&mut self, square: PySquare) -> Option<PyPiece> {
        let piece = self.piece_at(crate::util::PySquare(square.0));
        self.by_role.as_mut().for_each(|r| r.discard(square.0));
        self.by_color.as_mut().for_each(|c| c.discard(square.0));
        self.promoted.discard(square.0);
        piece
    }

    #[pyo3(signature = (square, piece, promoted=false))]
    pub fn set_piece_at(&mut self, square: PySquare, piece: Option<PyPiece>, promoted: bool) {
        self.remove_piece_at(square);
        if let Some(p) = piece {
            self.by_role.get_mut(p.0.role).add(square.0);
            self.by_color.get_mut(p.0.color).add(square.0);
            if promoted {
                self.promoted.add(square.0);
            }
        }
    }

    #[pyo3(signature = (promoted=None))]
    fn board_fen(&self, promoted: Option<bool>) -> PyResult<String> {
        self.board()?
            .board_fen_with_promoted(if promoted.unwrap_or(false) {
                self.promoted
            } else {
                Bitboard(0)
            })
            .map_err(|e| PyValueError::new_err(format!("Couldn't produce FEN, error: {e:?}")))
            .map(|x| x.to_string())
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
            if ((self.by_color.white | self.by_color.black).0 & mask) == 0 {
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

    pub fn apply_transform(&mut self, f: &Bound<'_, PyAny>) -> PyResult<()> {
        let apply = |bb: Bitboard| -> PyResult<Bitboard> {
            Ok(Bitboard(f.call1((bb.0,))?.extract::<u64>()?))
        };

        for role in self.by_role.as_mut() {
            *role = apply(*role)?;
        }
        for color in self.by_color.as_mut() {
            *color = apply(*color)?;
        }
        self.promoted = apply(self.promoted)?;
        Ok(())
    }

    fn transform(&self, f: &Bound<'_, PyAny>) -> PyResult<Self> {
        let mut board = self.clone();
        board.apply_transform(f)?;
        Ok(board)
    }

    fn mirror(&self) -> Self {
        let mut base_board = self.clone();
        base_board
            .by_role
            .as_mut()
            .for_each(|r| *r = r.flip_vertical());
        base_board
            .by_color
            .as_mut()
            .for_each(|c| *c = c.flip_vertical());
        base_board.promoted = base_board.promoted.flip_vertical();
        base_board
    }
}

impl BaseBoard {
    pub fn empty() -> PyResult<Self> {
        let (roles, colors) = Board::empty().into_bitboards();
        Ok(Self {
            by_role: roles,
            by_color: colors,
            promoted: Bitboard::EMPTY,
        })
    }

    pub fn set_occupied_w(&mut self, value: u64) {
        self.by_color.white = Bitboard(value);
    }
    pub fn set_occupied_b(&mut self, value: u64) {
        self.by_color.black = Bitboard(value);
    }

    pub fn occupied(&self) -> Bitboard {
        self.by_color.white | self.by_color.black
    }

    pub fn attackers_mask(
        &self,
        color: PyColor,
        square: PySquare,
        occupied: Option<Bitboard>,
    ) -> PyResult<Bitboard> {
        Ok(self.board()?.attacks_to(
            square.0,
            color.0,
            occupied.unwrap_or_else(|| self.occupied()),
        ))
    }
}

impl BaseBoard {
    pub fn attacks_mask(&self, square: shakmaty::Square) -> Bitboard {
        let occ = self.by_color.white | self.by_color.black;
        let role = self.by_role.find(|r| r.contains(square));
        let color = self.by_color.find(|c| c.contains(square));
        if let (Some(r), Some(c)) = (role, color) {
            shakmaty::attacks::attacks(square, Piece { role: r, color: c }, occ)
        } else {
            Bitboard::EMPTY
        }
    }
}

impl BaseBoard {
    pub fn apply_mirror(&mut self, py: Python<'_>) -> PyResult<()> {
        let flip_vertical = py.import("chess")?.getattr("flip_vertical")?;
        self.apply_transform(&flip_vertical)?;
        let white = self.by_color.white;
        let black = self.by_color.black;
        self.by_color.white = black;
        self.by_color.black = white;
        Ok(())
    }

    pub fn clear_board(&mut self) {
        let (roles, colors) = shakmaty::Board::empty().into_bitboards();
        self.by_role = roles;
        self.by_color = colors;
        self.promoted = shakmaty::Bitboard(0);
    }

    pub fn reset_board(&mut self) {
        let (roles, colors) = shakmaty::Board::new().into_bitboards();
        self.by_role = roles;
        self.by_color = colors;
        self.promoted = shakmaty::Bitboard(0);
    }
}

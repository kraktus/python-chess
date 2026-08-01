// All the code here is forked from shakmaty to address the fact python-chess has more detailed errors

use shakmaty::{
    Bitboard, Board, Castles, CastlingMode, Color, EnPassant, Rank, Role, Setup, Square, attacks,
};

use pyo3::prelude::*;

use bitflags::bitflags;

bitflags! {
    #[pyclass(from_py_object)]
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Status: u32 {
    const VALID = 0;
    const NO_WHITE_KING = 1 << 0;
    const NO_BLACK_KING = 1 << 1;
    const TOO_MANY_KINGS = 1 << 2;
    const TOO_MANY_WHITE_PAWNS = 1 << 3;
    const TOO_MANY_BLACK_PAWNS = 1 << 4;
    const PAWNS_ON_BACKRANK = 1 << 5;
    const TOO_MANY_WHITE_PIECES = 1 << 6;
    const TOO_MANY_BLACK_PIECES = 1 << 7;
    const BAD_CASTLING_RIGHTS = 1 << 8;
    const INVALID_EP_SQUARE = 1 << 9;
    const OPPOSITE_CHECK = 1 << 10;
    const EMPTY = 1 << 11;

    const RACE_CHECK = 1 << 12; // unused, only std supported
    const RACE_OVER = 1 << 13; // unused, only std supported
    const RACE_MATERIAL = 1 << 14; // unused, only std supported

    const TOO_MANY_CHECKERS = 1 << 15;
    const IMPOSSIBLE_CHECK = 1 << 16;
    }
}

#[pymethods]
impl Status {
    #[classattr]
    const STATUS_VALID: Self = Self::VALID;
    #[classattr]
    const STATUS_NO_WHITE_KING: Self = Self::NO_WHITE_KING;
    #[classattr]
    const STATUS_NO_BLACK_KING: Self = Self::NO_BLACK_KING;
    #[classattr]
    const STATUS_TOO_MANY_KINGS: Self = Self::TOO_MANY_KINGS;
    #[classattr]
    const STATUS_TOO_MANY_WHITE_PAWNS: Self = Self::TOO_MANY_WHITE_PAWNS;
    #[classattr]
    const STATUS_TOO_MANY_BLACK_PAWNS: Self = Self::TOO_MANY_BLACK_PAWNS;
    #[classattr]
    const STATUS_PAWNS_ON_BACKRANK: Self = Self::PAWNS_ON_BACKRANK;
    #[classattr]
    const STATUS_TOO_MANY_WHITE_PIECES: Self = Self::TOO_MANY_WHITE_PIECES;
    #[classattr]
    const STATUS_TOO_MANY_BLACK_PIECES: Self = Self::TOO_MANY_BLACK_PIECES;
    #[classattr]
    const STATUS_BAD_CASTLING_RIGHTS: Self = Self::BAD_CASTLING_RIGHTS;
    #[classattr]
    const STATUS_INVALID_EP_SQUARE: Self = Self::INVALID_EP_SQUARE;
    #[classattr]
    const STATUS_OPPOSITE_CHECK: Self = Self::OPPOSITE_CHECK;
    #[classattr]
    const STATUS_EMPTY: Self = Self::EMPTY;
    #[classattr]
    const STATUS_RACE_CHECK: Self = Self::RACE_CHECK;
    #[classattr]
    const STATUS_RACE_OVER: Self = Self::RACE_OVER;
    #[classattr]
    const STATUS_RACE_MATERIAL: Self = Self::RACE_MATERIAL;
    #[classattr]
    const STATUS_TOO_MANY_CHECKERS: Self = Self::TOO_MANY_CHECKERS;
    #[classattr]
    const STATUS_IMPOSSIBLE_CHECK: Self = Self::IMPOSSIBLE_CHECK;

    pub fn __eq__(&self, other: &Self) -> bool {
        self == other
    }

    pub fn __and__(&self, other: &Self) -> Self {
        self.intersection(*other)
    }

    pub fn __rand__(&self, other: &Self) -> Self {
        self.intersection(*other)
    }

    pub fn __or__(&self, other: &Self) -> Self {
        self.union(*other)
    }

    pub fn __ror__(&self, other: &Self) -> Self {
        self.union(*other)
    }

    pub fn __xor__(&self, other: &Self) -> Self {
        Self::from_bits_retain(self.bits() ^ other.bits())
    }

    pub fn __rxor__(&self, other: &Self) -> Self {
        Self::from_bits_retain(self.bits() ^ other.bits())
    }

    pub fn __invert__(&self) -> Self {
        Self::from_bits_retain(!self.bits())
    }

    pub fn __bool__(&self) -> bool {
        !self.is_empty()
    }

    pub fn __int__(&self) -> u32 {
        self.bits()
    }

    pub fn __repr__(&self) -> String {
        let bits = self.bits();
        let mut names: Vec<String> = Vec::new();
        let mut known = 0u32;
        for (name, flag) in self.iter_names() {
            names.push(name.to_owned());
            known |= flag.bits();
        }
        let extra = bits & !known;
        if names.is_empty() {
            if bits == 0 {
                return "<Status.VALID: 0>".to_string();
            }
            return format!("<Status: {bits}>");
        }
        let mut body = names.join("|");
        if extra != 0 {
            body.push_str(&format!("|{extra}"));
        }
        format!("<Status.{body}: {bits}>")
    }

    pub fn __str__(&self) -> String {
        format!("{}", self.bits())
    }
}

// python-chess considers castling rights valid if at least one king is on e1/e8,
// whereas shakmaty requires a unique king (`king_of()`).
// Example: "RNBKKBNR w KQkq - 0 1" has 2 white kings (e1, f1). Shakmaty strips castling
// rights to 0 causing false BAD_CASTLING_RIGHTS, but python-chess preserves 'KQ'.
fn clean_castling_rights(setup: &Setup, mode: CastlingMode) -> Bitboard {
    let castling = setup.castling_rights & setup.board.rooks();
    let mut white_castling = castling & Rank::First & setup.board.white();
    let mut black_castling = castling & Rank::Eighth & setup.board.black();

    if mode == CastlingMode::Standard {
        white_castling &= Bitboard::from(Square::A1) | Bitboard::from(Square::H1);
        black_castling &= Bitboard::from(Square::A8) | Bitboard::from(Square::H8);

        if (setup.board.white() & setup.board.kings() & Bitboard::from(Square::E1)).is_empty() {
            white_castling = Bitboard::EMPTY;
        }
        if (setup.board.black() & setup.board.kings() & Bitboard::from(Square::E8)).is_empty() {
            black_castling = Bitboard::EMPTY;
        }

        white_castling | black_castling
    } else {
        Castles::from_setup(setup, mode)
            .map_or_else(|c| c.castling_rights(), |c| c.castling_rights())
    }
}

// from shakmaty, renamed from Chess::from_setup_unchecked
pub fn status(setup: Setup, mode: CastlingMode) -> Status {
    let mut errors = Status::empty();

    let clean_rights = clean_castling_rights(&setup, mode);

    if setup.castling_rights != clean_rights {
        errors |= Status::BAD_CASTLING_RIGHTS;
    }

    let ep = match EnPassant::from_setup(&setup) {
        Ok(e) => e,
        Err(()) => {
            errors |= Status::INVALID_EP_SQUARE;
            None
        }
    };

    let checked_setup = Setup {
        castling_rights: clean_rights,
        ep_square: ep.map(Into::into),
        ..setup
    };

    errors |= validate(&checked_setup, ep);

    errors
}

// from shakmaty
const fn is_standard_material(board: &Board, color: Color) -> bool {
    let our = board.by_color(color);
    let promoted_pieces = board
        .queens()
        .intersect_const(our)
        .count()
        .saturating_sub(1)
        + board.rooks().intersect_const(our).count().saturating_sub(2)
        + board
            .knights()
            .intersect_const(our)
            .count()
            .saturating_sub(2)
        + board
            .bishops()
            .intersect_const(our)
            .intersect_const(Bitboard::LIGHT_SQUARES)
            .count()
            .saturating_sub(1)
        + board
            .bishops()
            .intersect_const(our)
            .intersect_const(Bitboard::DARK_SQUARES)
            .count()
            .saturating_sub(1);
    board.pawns().intersect_const(our).count() + promoted_pieces <= 8
}

fn our(s: &Setup, role: Role) -> Bitboard /* FINAL */ {
    s.board.by_piece(role.of(s.turn))
}

/// Bitboard of pieces giving check.
fn checkers(s: &Setup) -> Bitboard /* FINAL */ {
    our(s, Role::King).first().map_or(Bitboard(0), |king| {
        king_attackers(s, king, !s.turn, s.board.occupied())
    })
}

/// Attacks that a king on `square` would have to deal with.
fn king_attackers(s: &Setup, square: Square, attacker: Color, occupied: Bitboard) -> Bitboard {
    s.board.attacks_to(square, attacker, occupied)
}

// from shakmaty
fn validate(pos: &Setup, ep_square: Option<EnPassant>) -> Status {
    let mut errors = Status::empty();

    if pos.board.occupied().is_empty() {
        errors |= Status::EMPTY;
    }

    if (pos.board.pawns() & Bitboard::BACKRANKS).any() {
        errors |= Status::PAWNS_ON_BACKRANK;
    }

    for color in Color::ALL {
        let kings = pos.board.kings() & pos.board.by_color(color);
        if kings.is_empty() {
            if color.is_white() {
                errors |= Status::NO_WHITE_KING;
            } else {
                errors |= Status::NO_BLACK_KING;
            };
        } else if kings.more_than_one() {
            errors |= Status::TOO_MANY_KINGS;
        }

        if !is_standard_material(&pos.board, color) {
            if color.is_white() {
                errors |= Status::TOO_MANY_WHITE_PIECES;
            } else {
                errors |= Status::TOO_MANY_BLACK_PIECES;
            }
        }
    }

    if let Some(their_king) = pos.board.king_of(!pos.turn)
        && king_attackers(pos, their_king, pos.turn, pos.board.occupied()).any()
    {
        errors |= Status::OPPOSITE_CHECK;
    }

    let checkers = checkers(pos);
    if let (Some(a), Some(b), Some(our_king)) = (
        checkers.first(),
        checkers.last(),
        pos.board.king_of(pos.turn),
    ) {
        if let Some(ep_square) = ep_square {
            // The pushed pawn must be the only checker, or it has uncovered
            // check by a single sliding piece.
            if a != b
                || (a != ep_square.pawn_pushed_to()
                    && king_attackers(
                        pos,
                        our_king,
                        !pos.turn,
                        pos.board
                            .occupied()
                            .without(ep_square.pawn_pushed_to())
                            .with(ep_square.pawn_pushed_from()),
                    )
                    .any())
            {
                errors |= Status::IMPOSSIBLE_CHECK;
            }
        } else {
            // There can be at most two checkers, and discovered checkers
            // cannot be aligned.
            if checkers.count() > 2 {
                errors |= Status::TOO_MANY_CHECKERS;
            }
            if a != b && (checkers.count() > 2 || attacks::aligned(a, our_king, b)) {
                errors |= Status::IMPOSSIBLE_CHECK;
            }
        }
    }

    // Multiple steppers cannot be checkers.
    if (checkers & pos.board.steppers()).more_than_one() {
        errors |= Status::IMPOSSIBLE_CHECK;
    }

    errors
}

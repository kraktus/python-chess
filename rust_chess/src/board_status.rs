// All the code here is forked from shakmaty to address the fact python-chess has more detailed errors

use pyo3::exceptions::{PyException, PyValueError};
use shakmaty::fen::Fen;
use shakmaty::san::SanPlus;
use shakmaty::uci::UciMove;
use shakmaty::{
    Bitboard, Board, ByColor, ByRole, Castles, CastlingMode, CastlingSide, Chess, Color, EnPassant,
    FromSetup, MoveList, Position, PseudoLegal, Role, Setup, Square, attacks,
};

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::str::FromStr;

use crate::IllegalMoveError;
use crate::base_board::BaseBoard;
use crate::py_move::PyMove;
use crate::util::{IntOrBool, PyColor, PyRole, PySquare};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple, PyType};

use bitflags::bitflags;
use shakmaty::PositionErrorKinds;

bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct Status: u32 {
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

// from shakmaty, renamed from Chess::from_setup_unchecked
fn status(setup: Setup, mode: CastlingMode) -> Status {
    let mut errors = Status::empty();

    let castling_rights = match Castles::from_setup(&setup, mode) {
        Ok(castles) => castles.castling_rights(),
        Err(castles) => {
            errors |= Status::BAD_CASTLING_RIGHTS;
            castles.castling_rights()
        }
    };

    let ep = match EnPassant::from_setup(&setup) {
        Ok(e) => e,
        Err(()) => {
            errors |= Status::INVALID_EP_SQUARE;
            None
        }
    };

    let checked_setup = Setup {
        castling_rights,
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

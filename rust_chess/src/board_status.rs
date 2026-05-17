// All the code here is forked from shakmaty to address the fact python-chess has more detailed errors



fn validate<P: Position>(pos: &P, ep_square: Option<EnPassant>) -> PositionErrorKinds {
    let mut errors = PositionErrorKinds::empty();

    if pos.board().occupied().is_empty() {
        errors |= PositionErrorKinds::EMPTY_BOARD;
    }

    if (pos.board().pawns() & Bitboard::BACKRANKS).any() {
        errors |= PositionErrorKinds::PAWNS_ON_BACKRANK;
    }

    for color in Color::ALL {
        let kings = pos.board().kings() & pos.board().by_color(color);
        if kings.is_empty() {
            errors |= PositionErrorKinds::MISSING_KING;
        } else if kings.more_than_one() {
            errors |= PositionErrorKinds::TOO_MANY_KINGS;
        }

        if !is_standard_material(pos.board(), color) {
            errors |= PositionErrorKinds::TOO_MUCH_MATERIAL;
        }
    }

    if let Some(their_king) = pos.board().king_of(!pos.turn())
        && pos
            .king_attackers(their_king, pos.turn(), pos.board().occupied())
            .any()
    {
        errors |= PositionErrorKinds::OPPOSITE_CHECK;
    }

    let checkers = pos.checkers();
    if let (Some(a), Some(b), Some(our_king)) = (
        checkers.first(),
        checkers.last(),
        pos.board().king_of(pos.turn()),
    ) {
        if let Some(ep_square) = ep_square {
            // The pushed pawn must be the only checker, or it has uncovered
            // check by a single sliding piece.
            if a != b
                || (a != ep_square.pawn_pushed_to()
                    && pos
                        .king_attackers(
                            our_king,
                            !pos.turn(),
                            pos.board()
                                .occupied()
                                .without(ep_square.pawn_pushed_to())
                                .with(ep_square.pawn_pushed_from()),
                        )
                        .any())
            {
                errors |= PositionErrorKinds::IMPOSSIBLE_CHECK;
            }
        } else {
            // There can be at most two checkers, and discovered checkers
            // cannot be aligned.
            if a != b && (checkers.count() > 2 || attacks::aligned(a, our_king, b)) {
                errors |= PositionErrorKinds::IMPOSSIBLE_CHECK;
            }
        }
    }

    // Multiple steppers cannot be checkers.
    if (checkers & pos.board().steppers()).more_than_one() {
        errors |= PositionErrorKinds::IMPOSSIBLE_CHECK;
    }

    errors
}
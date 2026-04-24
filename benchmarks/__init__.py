import chess
from rust_chess import sum_as_string



def sum_as_string_py(a: int, b: int):
    return str(a + b)


class RustCHessTestSuite:

    def time_sum_as_str_py(self):
        sum_as_string_py(5, 20)

    def time_sum_as_str_rust(self):
        sum_as_string(5, 20)

class OutcomeSuite:
    def setup(self):
        self.outcomes = [
            chess.Outcome(chess.Termination.CHECKMATE, chess.WHITE),
            chess.Outcome(chess.Termination.CHECKMATE, chess.BLACK),
            chess.Outcome(chess.Termination.STALEMATE, None),
        ]

    def time_result_white_wins(self):
        self.outcomes[0].result()

    def time_result_black_wins(self):
        self.outcomes[1].result()

    def time_result_draw(self):
        self.outcomes[2].result()


class PieceSuite:
    def setup(self):
        self.pieces = [
            chess.Piece(pt, color) for pt in chess.PIECE_TYPES for color in chess.COLORS
        ]
        self.symbols = [p.symbol() for p in self.pieces]

    def time_symbol(self):
        for p in self.pieces:
            p.symbol()

    def time_unicode_symbol(self):
        for p in self.pieces:
            p.unicode_symbol()

    def time_unicode_symbol_inverted(self):
        for p in self.pieces:
            p.unicode_symbol(invert_color=True)

    def time_repr_svg(self):
        for p in self.pieces:
            p._repr_svg_()

    def time_hash(self):
        for p in self.pieces:
            hash(p)

    def time_from_symbol(self):
        for s in self.symbols:
            chess.Piece.from_symbol(s)


class MoveSuite:
    def setup(self):
        self.uci_strings = ["e2e4", "a7a8q", "0000", "N@b3"]
        self.moves = [chess.Move.from_uci(uci) for uci in self.uci_strings]

    def time_from_uci_normal(self):
        chess.Move.from_uci("e2e4")

    def time_from_uci_promotion(self):
        chess.Move.from_uci("a7a8q")

    def time_from_uci_null(self):
        chess.Move.from_uci("0000")

    def time_from_uci_invalid(self):
        for uci in ["e2e9", "e", "a2a2", "Z@e4"]:
            try:
                chess.Move.from_uci(uci)
            except ValueError:
                pass

    def time_str_repr(self):
        for m in self.moves:
            str(m)
            repr(m)

    def time_uci(self):
        for m in self.moves:
            m.uci()

    def time_xboard(self):
        for m in self.moves:
            m.xboard()

    def time_bool(self):
        for m in self.moves:
            bool(m)

    def time_null_classmethod(self):
        for _ in range(10):
            chess.Move.null()


class BaseBoardSuite:
    def setup(self):
        self.board = chess.BaseBoard(chess.STARTING_BOARD_FEN)
        self.squares = list(chess.SQUARES)
        self.piece_map = self.board.piece_map()
        self.board_copy = self.board.copy()
        self.occupied = list(self.piece_map.keys())

    def time_reset_board(self):
        self.board.reset_board()

    def time_clear_board(self):
        self.board.clear_board()

    def time_piece_count(self):
        self.board.piece_count()

    def time_pieces_mask(self):
        self.board.pieces_mask(chess.PAWN, chess.WHITE)

    def time_pieces(self):
        self.board.pieces(chess.PAWN, chess.WHITE)

    def time_piece_at(self):
        for sq in self.squares:
            self.board.piece_at(sq)

    def time_piece_type_at(self):
        for sq in self.squares:
            self.board.piece_type_at(sq)

    def time_color_at(self):
        for sq in self.squares:
            self.board.color_at(sq)

    def time_king(self):
        self.board.king(chess.WHITE)
        self.board.king(chess.BLACK)

    def time_attacks_mask(self):
        for sq in self.occupied:
            self.board.attacks_mask(sq)

    def time_attacks(self):
        for sq in self.occupied:
            self.board.attacks(sq)

    def time_attackers_mask(self):
        for sq in self.squares:
            self.board.attackers_mask(chess.WHITE, sq)

    def time_is_attacked_by(self):
        for sq in self.squares:
            self.board.is_attacked_by(chess.WHITE, sq)

    def time_attackers(self):
        for sq in self.squares:
            self.board.attackers(chess.WHITE, sq)

    def time_pin_mask(self):
        for sq in self.squares:
            self.board.pin_mask(chess.WHITE, sq)
        # Empty board pin_mask
        chess.BaseBoard(None).pin_mask(chess.WHITE, chess.A1)

    def time_pin(self):
        self.board.pin(chess.WHITE, chess.E2)

    def time_pieces_mask_invalid(self):
        try:
            self.board.pieces_mask(99, chess.WHITE)
        except AssertionError:
            pass

    def time_init_custom_fen(self):
        chess.BaseBoard("8/8/8/8/8/8/8/8")

    def time_is_pinned(self):
        for sq in self.squares:
            self.board.is_pinned(chess.WHITE, sq)

    def time_board_fen(self):
        self.board.board_fen()
        self.board.board_fen(promoted=True)
        self.board.board_fen(promoted=False)

    def time_set_board_fen_invalid(self):
        for fen in [
            "8/8",
            "8/8/8/8/8/8/8/8 w",
            "8/8/8/8/8/8/8/8/8",
            "11/8/8/8/8/8/8/8",
            "Z/8/8/8/8/8/8/8",
            "p~p/8/8/8/8/8/8/8",
        ]:
            try:
                self.board.set_board_fen(fen)
            except ValueError:
                pass

    def time_set_chess960_pos_invalid(self):
        try:
            self.board.set_chess960_pos(1000)
        except ValueError:
            pass

    def time_unicode_options(self):
        self.board.unicode(borders=True)
        self.board.unicode(orientation=chess.BLACK)

    def time_repr_svg(self):
        self.board._repr_svg_()

    def time_set_board_fen(self):
        self.board.set_board_fen(chess.STARTING_BOARD_FEN)

    def time_piece_map(self):
        self.board.piece_map()

    def time_set_piece_map(self):
        self.board.set_piece_map(self.piece_map)

    def time_copy(self):
        self.board.copy()
        import copy

        copy.copy(self.board)
        copy.deepcopy(self.board)

    def time_mirror(self):
        self.board.mirror()

    def time_chess960_pos(self):
        self.board.chess960_pos()

    def time_set_chess960_pos(self):
        self.board.set_chess960_pos(518)

    def time_unicode(self):
        self.board.unicode()

    def time_str_repr(self):
        str(self.board)
        repr(self.board)

    def time_eq(self):
        return self.board == self.board_copy

    def time_apply_transform(self):
        self.board.apply_transform(chess.flip_vertical)

    def time_transform(self):
        self.board.transform(chess.flip_vertical)

    def time_apply_mirror(self):
        self.board.apply_mirror()

    def time_empty_classmethod(self):
        chess.BaseBoard.empty()

    def time_from_chess960_pos_classmethod(self):
        chess.BaseBoard.from_chess960_pos(518)

    def time_remove_set_piece(self):
        self.board.remove_piece_at(chess.E2)
        self.board.set_piece_at(chess.E2, None)
        self.board.set_piece_at(chess.E4, chess.Piece(chess.PAWN, chess.WHITE))


class BoardSuite:
    def setup(self):
        self.board = chess.Board(chess.STARTING_FEN)
        self.midgame_fen = (
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
        )
        self.midgame_board = chess.Board(self.midgame_fen)
        self.checkmate_fen = (
            "rnb1kbnr/pppp1ppp/4p3/8/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3"
        )
        self.checkmate_board = chess.Board(self.checkmate_fen)
        self.stalemate_fen = "k7/8/1Q6/8/8/8/8/7K b - - 0 1"
        self.stalemate_board = chess.Board(self.stalemate_fen)
        self.legal_moves = list(self.board.legal_moves)

        self.push_pop_board = chess.Board()
        self.push_pop_moves = list(self.push_pop_board.legal_moves)[:5]

        self.variation_moves = []
        b = chess.Board()
        for _ in range(5):
            move = list(b.legal_moves)[0]
            self.variation_moves.append(move)
            b.push(move)

        self.pseudo_legal_moves = list(self.board.pseudo_legal_moves)
        self.epd_string = self.board.epd()

    def time_reset(self):
        self.board.reset()

    def time_clear(self):
        self.board.clear()

    def time_ply(self):
        self.board.ply()

    def time_legal_moves_list(self):
        list(self.board.legal_moves)

    def time_legal_moves_count(self):
        self.board.legal_moves.count()

    def time_pseudo_legal_moves_list(self):
        list(self.board.pseudo_legal_moves)

    def time_generate_pseudo_legal_captures(self):
        list(self.board.generate_pseudo_legal_captures())

    def time_generate_legal_moves(self):
        list(self.board.generate_legal_moves())

    def time_generate_legal_captures(self):
        list(self.board.generate_legal_captures())

    def time_generate_castling_moves(self):
        list(self.midgame_board.generate_castling_moves())

    def time_checkers_mask(self):
        self.board.checkers_mask()

    def time_is_check(self):
        self.board.is_check()

    def time_is_checkmate(self):
        self.board.is_checkmate()

    def time_is_stalemate(self):
        self.board.is_stalemate()

    def time_is_insufficient_material(self):
        self.board.is_insufficient_material()

    def time_has_insufficient_material(self):
        self.board.has_insufficient_material(chess.WHITE)

    def time_is_game_over(self):
        self.board.is_game_over()

    def time_outcome(self):
        self.checkmate_board.outcome()

    def time_is_repetition(self):
        self.push_pop_board.is_repetition()

    def time_has_pseudo_legal_en_passant(self):
        self.board.has_pseudo_legal_en_passant()

    def time_has_legal_en_passant(self):
        self.board.has_legal_en_passant()

    def time_is_fifty_moves(self):
        self.board.is_fifty_moves()

    def time_is_fivefold_repetition(self):
        self.board.is_fivefold_repetition()

    def time_is_seventyfive_moves(self):
        self.board.is_seventyfive_moves()

    def time_can_claim_fifty_moves(self):
        self.board.can_claim_fifty_moves()

    def time_can_claim_threefold_repetition(self):
        self.board.can_claim_threefold_repetition()

    def time_result(self):
        self.checkmate_board.result()

    def time_is_into_check(self):
        for move in self.pseudo_legal_moves:
            self.board.is_into_check(move)

    def time_was_into_check(self):
        self.board.was_into_check()

    def time_is_irreversible(self):
        for move in self.pseudo_legal_moves:
            self.board.is_irreversible(move)

    def time_is_zeroing(self):
        for move in self.pseudo_legal_moves:
            self.board.is_zeroing(move)

    def time_lan(self):
        for move in self.legal_moves:
            self.board.lan(move)

    def time_san_and_push(self):
        b = chess.Board()
        for _ in range(5):
            move = list(b.legal_moves)[0]
            b.san_and_push(move)

    def time_push_san(self):
        b = chess.Board()
        b.push_san("e4")

    def time_push_uci(self):
        b = chess.Board()
        b.push_uci("e2e4")

    def time_parse_xboard(self):
        self.board.parse_xboard("e4")

    def time_push_xboard(self):
        b = chess.Board()
        b.push_xboard("e4")

    def time_xboard(self):
        for move in self.legal_moves:
            self.board.xboard(move)

    def time_is_variant_end_etc(self):
        self.board.is_variant_end()
        self.board.is_variant_loss()
        self.board.is_variant_win()
        self.board.is_variant_draw()

    def time_can_claim_draw_full(self):
        # Setup a position to claim 50 moves
        b = chess.Board("8/8/8/8/8/8/8/8 w - - 99 1")
        b.can_claim_fifty_moves()
        b.can_claim_draw()
        # Setup a position to claim threefold repetition
        b = chess.Board()
        b.push_san("e4")
        b.push_san("e5")
        b.push_san("Nf3")
        b.push_san("Nc6")
        b.push_san("Ng1")
        b.push_san("Nb8")
        b.push_san("Nf3")
        b.push_san("Nc6")
        b.can_claim_threefold_repetition()

    def time_is_repetition_cache(self):
        b = chess.Board()
        for san in ["Nf3", "Nf6", "Ng1", "Ng8", "Nf3", "Nf6"]:
            b.push_san(san)
        b.is_repetition(3)

    def time_board_fen_promoted(self):
        b = chess.Board()
        b.set_piece_at(chess.E4, chess.Piece(chess.QUEEN, chess.WHITE), promoted=True)
        b.board_fen(promoted=True)
        b.board_fen(promoted=False)
        try:
            b.set_board_fen("Q~7/8/8/8/8/8/8/8")
        except ValueError:
            pass

    def time_eq_not_implemented(self):
        return self.board == 1

    def time_root_no_stack(self):
        chess.Board().root()

    def time_remove_empty(self):
        self.board.remove_piece_at(chess.A3)

    def time_set_piece_at(self):
        self.board.set_piece_at(chess.A3, chess.Piece(chess.PAWN, chess.WHITE))
        self.board._set_piece_at(chess.A3, 99, chess.WHITE)  # Invalid piece type

    def time_generate_promotions(self):
        b = chess.Board("8/P7/8/8/8/8/8/8 w - - 0 1")
        list(b.generate_pseudo_legal_moves())
        b = chess.Board("8/8/8/8/8/8/p7/8 b - - 0 1")
        list(b.generate_pseudo_legal_moves())

    def time_generate_ep_blocked(self):
        b = chess.Board("8/8/8/3pP3/8/8/8/8 w - d6 0 1")
        b.set_piece_at(
            chess.D6, chess.Piece(chess.KNIGHT, chess.WHITE)
        )  # Block the ep square
        list(b.generate_pseudo_legal_moves())

    def time_chess960_pos_none(self):
        b = chess.Board()
        b.push_san("e4")
        b.chess960_pos()
        b = chess.Board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
        b.set_piece_at(chess.E2, None)
        b.chess960_pos()

    def time_gives_checkmate(self):
        m = self.checkmate_board.legal_moves
        if m:
            self.checkmate_board.gives_checkmate(list(m)[0])

    def time_invalid_char_fen(self):
        try:
            chess.BaseBoard().set_board_fen("Z7/8/8/8/8/8/8/8")
        except ValueError:
            pass

    def time_push_null_drop_promote(self):
        b = chess.Board("8/P7/8/8/8/8/8/8 w - - 0 1")
        b.push(chess.Move.null())
        b.push(chess.Move.from_uci("P@a5"))
        b = chess.Board("8/P7/8/8/8/8/8/8 w - - 0 1")
        b.push(chess.Move.from_uci("a7a8q"))

    def time_push_castling_rights(self):
        b = chess.Board()
        b.push(chess.Move.from_uci("e2e4"))
        b.push(chess.Move.from_uci("h8h7"))
        b.push(chess.Move.from_uci("a1a2"))
        b.push(chess.Move.from_uci("e8e7"))

    def time_push_castling(self):
        b = chess.Board("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1")
        b.push_san("O-O")
        b = chess.Board("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1")
        b.push_san("O-O-O")

    def time_set_fen_errors(self):
        for f in [
            "8/8/8/8/8/8/8/8 x",
            "8/8/8/8/8/8/8/8 w x",
            "8/8/8/8/8/8/8/8 w - - x",
            "8/8/8/8/8/8/8/8 w - - 0 x",
        ]:
            try:
                self.board.set_fen(f)
            except ValueError:
                pass

    def time_fen_options(self):
        self.board.fen(en_passant="fen")
        self.board.fen(en_passant="xfen")
        self.board.fen(promoted=True)

    def time_epd_options(self):
        self.board.epd(hmvc=self.board.halfmove_clock, fmvn=self.board.fullmove_number)
        self.board.epd(en_passant="fen")
        self.board.epd(en_passant="xfen")
        self.board.epd(pv=[chess.Move.from_uci("e2e4")])
        self.board.epd(am=list(self.board.legal_moves)[:2])

    def time_set_epd(self):
        self.board.set_epd(
            'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - hmvc 0; fmvn 1; id "test"; c0 "escaped\\nstring"; v 0.5; pv e2e4 e7e5; bm e2e4;'
        )

    def time_parse_san_castling(self):
        b = chess.Board("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1")
        b.parse_san("O-O")
        b.parse_san("O-O-O")
        try:
            b.parse_san("O-O-O-O")
        except ValueError:
            pass

    def time_parse_san_disambiguation(self):
        b = chess.Board("4k3/8/8/8/8/8/N1N5/4K3 w - - 0 1")
        b.parse_san("Nac3")
        b.parse_san("Ncc3")
        try:
            b.parse_san("Nc3")
        except ValueError:
            pass

    def time_variant_checks(self):
        self.board.is_variant_end()
        self.board.is_variant_loss()
        self.board.is_variant_win()
        self.board.is_variant_draw()

    def time_push_pop(self):
        for move in self.push_pop_moves:
            self.push_pop_board.push(move)
            self.push_pop_board.pop()

    def time_gives_check(self):
        for move in self.legal_moves:
            self.board.gives_check(move)

    def time_is_legal(self):
        for move in self.pseudo_legal_moves:
            self.board.is_legal(move)

    def time_is_pseudo_legal(self):
        for move in self.pseudo_legal_moves:
            self.board.is_pseudo_legal(move)

    def time_find_move(self):
        self.board.find_move(chess.E2, chess.E4)

    def time_find_move_promotion(self):
        board = chess.Board("8/P7/8/8/8/8/8/8 w - - 0 1")
        board.find_move(chess.A7, chess.A8)

    def time_peek(self):
        self.board.push(chess.Move.from_uci("e2e4"))
        self.board.peek()

    def time_castling_shredder_fen(self):
        self.board.castling_shredder_fen()
        self.board_chess960.castling_shredder_fen()

    def time_castling_xfen(self):
        self.board.castling_xfen()
        self.board_chess960.castling_xfen()
        board_complex = chess.Board(
            "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", chess960=True
        )
        board_complex.castling_xfen()

    def time_is_en_passant(self):
        for move in self.midgame_board.pseudo_legal_moves:
            self.midgame_board.is_en_passant(move)

    def time_is_capture(self):
        for move in self.midgame_board.pseudo_legal_moves:
            self.midgame_board.is_capture(move)

    def time_is_castling(self):
        for move in self.midgame_board.pseudo_legal_moves:
            self.midgame_board.is_castling(move)

    def time_has_chess960_castling_rights(self):
        self.board.has_chess960_castling_rights()
        self.board_chess960.has_chess960_castling_rights()

    def time_clean_castling_rights(self):
        self.board.clean_castling_rights()
        self.board_chess960.clean_castling_rights()

    def time_is_kingside_castling(self):
        for move in self.midgame_board.pseudo_legal_moves:
            self.midgame_board.is_kingside_castling(move)

    def time_is_queenside_castling(self):
        for move in self.midgame_board.pseudo_legal_moves:
            self.midgame_board.is_queenside_castling(move)

    def time_is_repetition_count(self):
        self.board.is_repetition(2)
        self.midgame_board.is_repetition(3)

    def time_generate_pseudo_legal_moves_masks(self):
        list(
            self.board.generate_pseudo_legal_moves(
                chess.BB_A2 | chess.BB_B2, chess.BB_A3 | chess.BB_A4
            )
        )
        list(
            self.midgame_board.generate_pseudo_legal_captures(chess.BB_E4, chess.BB_D5)
        )
        list(
            self.board.generate_legal_moves(
                chess.BB_A2 | chess.BB_B2, chess.BB_A3 | chess.BB_A4
            )
        )
        list(self.midgame_board.generate_legal_captures(chess.BB_E4, chess.BB_D5))

    def time_status(self):
        self.board.status()

    def time_is_valid(self):
        self.board.is_valid()

    def time_copy_with_stack(self):
        self.board.copy(stack=True)

    def time_copy_no_stack(self):
        self.board.copy(stack=False)

    def time_mirror(self):
        self.board.mirror()

    def time_root(self):
        self.board.root()


class PseudoLegalMoveGeneratorSuite:
    def setup(self):
        self.board = chess.Board(chess.STARTING_FEN)
        self.legal_move = list(self.board.legal_moves)[0]
        self.illegal_move = chess.Move.from_uci("e2e5")

    def get_gen(self):
        return self.board.pseudo_legal_moves

    def time_bool(self):
        bool(self.get_gen())

    def time_count(self):
        self.get_gen().count()

    def time_iter(self):
        list(self.get_gen())

    def time_contains_legal(self):
        self.legal_move in self.get_gen()

    def time_contains_illegal(self):
        self.illegal_move in self.get_gen()


class LegalMoveGeneratorSuite:
    def setup(self):
        self.board = chess.Board(chess.STARTING_FEN)
        self.legal_move = list(self.board.legal_moves)[0]
        self.illegal_move = chess.Move.from_uci("e2e5")

    def get_gen(self):
        return self.board.legal_moves

    def time_bool(self):
        bool(self.get_gen())

    def time_count(self):
        self.get_gen().count()

    def time_iter(self):
        list(self.get_gen())

    def time_contains_legal(self):
        self.legal_move in self.get_gen()

    def time_contains_illegal(self):
        self.illegal_move in self.get_gen()


class SquareSetSuite:
    def setup(self):
        self.empty = chess.SquareSet(chess.BB_EMPTY)
        self.full = chess.SquareSet(chess.BB_ALL)
        self.rank = chess.SquareSet.ray(chess.A1, chess.H1)
        self.file_ = chess.SquareSet.ray(chess.A1, chess.A8)
        self.diagonal = chess.SquareSet.ray(chess.A1, chess.H8)
        self.sparse = chess.SquareSet(0x10204081020408)
        self.squares = list(chess.SQUARES)
        self.A1 = chess.A1
        self.H8 = chess.H8
        self.E4 = chess.E4

    def time_contains(self):
        for sq in self.squares:
            sq in self.full

    def time_iter(self):
        list(self.full)

    def time_len(self):
        len(self.full)

    def time_add(self):
        s = chess.SquareSet()
        for sq in self.squares:
            s.add(sq)

    def time_discard(self):
        s = chess.SquareSet(chess.BB_ALL)
        for sq in self.squares:
            s.discard(sq)

    def time_union(self):
        self.rank | self.file_

    def time_intersection(self):
        self.rank & self.file_

    def time_difference(self):
        self.full - self.sparse

    def time_symmetric_difference(self):
        self.rank ^ self.file_

    def time_issubset(self):
        self.rank.issubset(self.full)

    def time_issuperset(self):
        self.full.issuperset(self.rank)

    def time_isdisjoint(self):
        self.rank.isdisjoint(self.file_)

    def time_carry_rippler(self):
        list(self.sparse.carry_rippler())

    def time_mirror(self):
        self.sparse.mirror()

    def time_tolist(self):
        self.sparse.tolist()

    def time_invert(self):
        ~self.sparse

    def time_lshift(self):
        self.sparse << 8

    def time_rshift(self):
        self.sparse >> 8

    def time_ray(self):
        chess.SquareSet.ray(self.A1, self.H8)

    def time_between(self):
        chess.SquareSet.between(self.A1, self.H8)

    def time_from_square(self):
        chess.SquareSet.from_square(self.E4)

    def time_int(self):
        int(self.sparse)

    def time_index(self):
        import operator

        operator.index(self.sparse)

    def time_str_repr(self):
        str(self.sparse)
        repr(self.sparse)

    def time_update_methods(self):
        s = self.sparse.copy()
        s.update(self.rank)
        s.intersection_update(self.rank)
        s.difference_update(self.rank)
        s.symmetric_difference_update(self.rank)

    def time_i_methods(self):
        s = self.sparse.copy()
        s |= self.rank
        s &= self.rank
        s -= self.rank
        s ^= self.rank
        s <<= 1
        s >>= 1

    def time_pop(self):
        s = self.sparse.copy()
        while s:
            s.pop()

    def time_remove(self):
        s = self.sparse.copy()
        for sq in list(s):
            s.remove(sq)

    def time_clear(self):
        s = self.sparse.copy()
        s.clear()

    def time_bool(self):
        bool(self.sparse)

    def time_eq(self):
        return self.sparse == self.full


class GlobalFunctionSuite:
    def setup(self):
        self.squares = list(chess.SQUARES)
        self.square_names = [chess.square_name(sq) for sq in self.squares]
        self.files = list(chess.FILES)
        self.ranks = list(chess.RANKS)
        self.file_names = [chess.file_name(f) for f in self.files]
        self.rank_names = [chess.rank_name(r) for r in self.ranks]
        self.piece_types = list(chess.PIECE_TYPES)

    def time_piece_symbol(self):
        for pt in self.piece_types:
            chess.piece_symbol(pt)

    def time_piece_name(self):
        for pt in self.piece_types:
            chess.piece_name(pt)

    def time_parse_square(self):
        for name in self.square_names:
            chess.parse_square(name)

    def time_square_name(self):
        for sq in self.squares:
            chess.square_name(sq)

    def time_square(self):
        for f in self.files:
            for r in self.ranks:
                chess.square(f, r)

    def time_parse_file(self):
        for name in self.file_names:
            chess.parse_file(name)

    def time_file_name(self):
        for f in self.files:
            chess.file_name(f)

    def time_parse_rank(self):
        for name in self.rank_names:
            chess.parse_rank(name)

    def time_rank_name(self):
        for r in self.ranks:
            chess.rank_name(r)

    def time_square_file(self):
        for sq in self.squares:
            chess.square_file(sq)

    def time_square_rank(self):
        for sq in self.squares:
            chess.square_rank(sq)

    def time_square_distance(self):
        for sq in self.squares:
            chess.square_distance(chess.A1, sq)

    def time_square_manhattan_distance(self):
        for sq in self.squares:
            chess.square_manhattan_distance(chess.A1, sq)

    def time_square_knight_distance(self):
        for sq in self.squares:
            chess.square_knight_distance(chess.A1, sq)

    def time_square_mirror(self):
        for sq in self.squares:
            chess.square_mirror(sq)


class BitboardSuite:
    def setup(self):
        self.bitboards = [
            chess.BB_EMPTY,
            chess.BB_ALL,
            chess.BB_RANK_1,
            chess.BB_FILE_A,
            0x8040201008040201,
            0x0102040810204080,
            0x10204081020408,
        ]
        self.squares = list(chess.SQUARES)

    def time_lsb(self):
        for bb in self.bitboards[1:]:
            chess.lsb(bb)

    def time_scan_forward(self):
        for bb in self.bitboards:
            list(chess.scan_forward(bb))

    def time_msb(self):
        for bb in self.bitboards[1:]:
            chess.msb(bb)

    def time_scan_reversed(self):
        for bb in self.bitboards:
            list(chess.scan_reversed(bb))

    def time_flip_vertical(self):
        for bb in self.bitboards:
            chess.flip_vertical(bb)

    def time_flip_horizontal(self):
        for bb in self.bitboards:
            chess.flip_horizontal(bb)

    def time_flip_diagonal(self):
        for bb in self.bitboards:
            chess.flip_diagonal(bb)

    def time_flip_anti_diagonal(self):
        for bb in self.bitboards:
            chess.flip_anti_diagonal(bb)

    def time_shift_down(self):
        for bb in self.bitboards:
            chess.shift_down(bb)

    def time_shift_2_down(self):
        for bb in self.bitboards:
            chess.shift_2_down(bb)

    def time_shift_up(self):
        for bb in self.bitboards:
            chess.shift_up(bb)

    def time_shift_2_up(self):
        for bb in self.bitboards:
            chess.shift_2_up(bb)

    def time_shift_right(self):
        for bb in self.bitboards:
            chess.shift_right(bb)

    def time_shift_2_right(self):
        for bb in self.bitboards:
            chess.shift_2_right(bb)

    def time_shift_left(self):
        for bb in self.bitboards:
            chess.shift_left(bb)

    def time_shift_2_left(self):
        for bb in self.bitboards:
            chess.shift_2_left(bb)

    def time_shift_up_left(self):
        for bb in self.bitboards:
            chess.shift_up_left(bb)

    def time_shift_up_right(self):
        for bb in self.bitboards:
            chess.shift_up_right(bb)

    def time_shift_down_left(self):
        for bb in self.bitboards:
            chess.shift_down_left(bb)

    def time_shift_down_right(self):
        for bb in self.bitboards:
            chess.shift_down_right(bb)

    def time_ray(self):
        for sq in self.squares:
            chess.ray(chess.E4, sq)

    def time_between(self):
        for sq in self.squares:
            chess.between(chess.E4, sq)

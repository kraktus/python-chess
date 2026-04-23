import chess

class Suite:
    def setup(self):
        self.piece_symbols = [x for p in ["p", "k", "b", "r", "q", "k"] for x in [p, p.upper()]]

    def time_piece_from_symbol(self):
        for p in self.piece_symbols:
            chess.Piece.from_symbol(p)

    def time_baseboard_from_fen(self):
        chess.BaseBoard(board_fen="8/8/8/8/8/8/8/8")
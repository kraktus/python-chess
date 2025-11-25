import chess
import struct
from typing import Tuple, Optional, List

class BinaryFen:
    """
    A class for encoding and decoding chess positions to and from a compact binary format.
    """

    def __init__(self, value: bytes):
        self.value = value

    def read(self):
        """
        Decodes the binary FEN into a (board, fullmove_number) tuple.
        Returns:
            (chess.Board, int): The decoded board and fullmove_number.
        """
        reader = _ByteReader(self.value)
        occupied = _read_long(reader)
        occupied_bb = chess.SquareSet(occupied)

        pawns = 0
        knights = 0
        bishops = 0
        rooks = 0
        queens = 0
        kings = 0
        white = 0
        black = 0

        turn = chess.WHITE
        unmoved_rooks = 0
        ep_move = None

        def unpack_piece(sq, nibble):
            nonlocal pawns, knights, bishops, rooks, queens, kings, white, black, unmoved_rooks, ep_move, turn
            bb = 1 << sq
            if nibble == 0:
                pawns |= bb
                white |= bb
            elif nibble == 1:
                pawns |= bb
                black |= bb
            elif nibble == 2:
                knights |= bb
                white |= bb
            elif nibble == 3:
                knights |= bb
                black |= bb
            elif nibble == 4:
                bishops |= bb
                white |= bb
            elif nibble == 5:
                bishops |= bb
                black |= bb
            elif nibble == 6:
                rooks |= bb
                white |= bb
            elif nibble == 7:
                rooks |= bb
                black |= bb
            elif nibble == 8:
                queens |= bb
                white |= bb
            elif nibble == 9:
                queens |= bb
                black |= bb
            elif nibble == 10:
                kings |= bb
                white |= bb
            elif nibble == 11:
                kings |= bb
                black |= bb
            elif nibble == 12:
                pawns |= bb
                # ep_move: from square is sq ^ 0x10, to square is sq
                ep_move = (sq ^ 0x10, sq)
                if chess.square_rank(sq) <= chess.RANK_4:
                    white |= bb
                else:
                    black |= bb
            elif nibble == 13:
                rooks |= bb
                white |= bb
                unmoved_rooks |= bb
            elif nibble == 14:
                rooks |= bb
                black |= bb
                unmoved_rooks |= bb
            elif nibble == 15:
                kings |= bb
                black |= bb
                turn = chess.BLACK

        occ_squares = [sq for sq in range(64) if (occupied >> sq) & 1]
        i = 0
        while i < len(occ_squares):
            lo, hi = _read_nibbles(reader)
            unpack_piece(occ_squares[i], lo)
            i += 1
            if i < len(occ_squares):
                unpack_piece(occ_squares[i], hi)
                i += 1

        # Read halfmove clock and ply (fullmove number)
        halfmove_clock = 0
        ply = 1
        try:
            halfmove_clock = _read_leb128(reader)
            ply = _read_leb128(reader)
        except StopIteration:
            pass

        # Variant and other fields are not handled in this minimal port
        # For now, just create a standard chess.Board
        board = chess.Board(None)
        # Set pieces
        for sq in range(64):
            bb = 1 << sq
            if pawns & bb:
                color = chess.WHITE if white & bb else chess.BLACK
                board.set_piece_at(sq, chess.Piece(chess.PAWN, color))
            elif knights & bb:
                color = chess.WHITE if white & bb else chess.BLACK
                board.set_piece_at(sq, chess.Piece(chess.KNIGHT, color))
            elif bishops & bb:
                color = chess.WHITE if white & bb else chess.BLACK
                board.set_piece_at(sq, chess.Piece(chess.BISHOP, color))
            elif rooks & bb:
                color = chess.WHITE if white & bb else chess.BLACK
                board.set_piece_at(sq, chess.Piece(chess.ROOK, color))
            elif queens & bb:
                color = chess.WHITE if white & bb else chess.BLACK
                board.set_piece_at(sq, chess.Piece(chess.QUEEN, color))
            elif kings & bb:
                color = chess.WHITE if white & bb else chess.BLACK
                board.set_piece_at(sq, chess.Piece(chess.KING, color))
            else:
                board.remove_piece_at(sq)

        board.turn = turn
        board.halfmove_clock = halfmove_clock
        board.fullmove_number = (ply + 1) // 2 if ply > 0 else 1

        # Set en passant square if present
        if ep_move is not None:
            board.ep_square = ep_move[1]
        else:
            board.ep_square = None

        # TODO: set castling rights, variant, etc.

        return board, board.fullmove_number

    @staticmethod
    def write(board: chess.Board) -> "BinaryFen":
        """
        Encodes a chess.Board into a BinaryFen.
        """
        builder = bytearray()
        # Occupied bitboard
        occupied = int(board.occupied)
        _write_long(builder, occupied)

        # Gather unmoved rooks (not tracked in python-chess, so just 0)
        unmoved_rooks = 0

        # Pack pieces
        occ_squares = [sq for sq in range(64) if (occupied >> sq) & 1]
        def pack_piece(sq):
            piece = board.piece_at(sq)
            if piece is None:
                return 0
            if piece.piece_type == chess.PAWN:
                # En passant not handled
                return 0 if piece.color == chess.WHITE else 1
            elif piece.piece_type == chess.KNIGHT:
                return 2 if piece.color == chess.WHITE else 3
            elif piece.piece_type == chess.BISHOP:
                return 4 if piece.color == chess.WHITE else 5
            elif piece.piece_type == chess.ROOK:
                return 6 if piece.color == chess.WHITE else 7
            elif piece.piece_type == chess.QUEEN:
                return 8 if piece.color == chess.WHITE else 9
            elif piece.piece_type == chess.KING:
                if piece.color == chess.WHITE:
                    return 10
                else:
                    return 11 if board.turn == chess.WHITE else 15
            return 0

        i = 0
        while i < len(occ_squares):
            lo = pack_piece(occ_squares[i])
            i += 1
            hi = pack_piece(occ_squares[i]) if i < len(occ_squares) else 0
            _write_nibbles(builder, lo, hi)
            i += 1

        # Write halfmove clock and ply
        _write_leb128(builder, board.halfmove_clock)
        ply = 2 * (board.fullmove_number - 1) + (0 if board.turn == chess.WHITE else 1)
        _write_leb128(builder, ply)

        # Variant and other fields not handled

        return BinaryFen(bytes(builder))

    def __eq__(self, other):
        if not isinstance(other, BinaryFen):
            return False
        return self.value == other.value

    def __hash__(self):
        return hash(self.value)

# Helper classes and functions

class _ByteReader:
    def __init__(self, data: bytes):
        self.data = data
        self.pos = 0

    def next(self):
        if self.pos >= len(self.data):
            raise StopIteration
        b = self.data[self.pos]
        self.pos += 1
        return b

def _read_long(reader: "_ByteReader") -> int:
    return (
        (reader.next() << 56) |
        (reader.next() << 48) |
        (reader.next() << 40) |
        (reader.next() << 32) |
        (reader.next() << 24) |
        (reader.next() << 16) |
        (reader.next() << 8) |
        (reader.next())
    )

def _write_long(builder: bytearray, v: int):
    builder.extend([
        (v >> 56) & 0xFF,
        (v >> 48) & 0xFF,
        (v >> 40) & 0xFF,
        (v >> 32) & 0xFF,
        (v >> 24) & 0xFF,
        (v >> 16) & 0xFF,
        (v >> 8) & 0xFF,
        v & 0xFF,
    ])

def _read_leb128(reader: "_ByteReader") -> int:
    n = 0
    shift = 0
    while True:
        b = reader.next()
        n |= (b & 127) << shift
        if (b & 128) == 0:
            break
        shift += 7
    return n

def _write_leb128(builder: bytearray, v: int):
    n = v
    while n > 127:
        builder.append((n | 128) & 0xFF)
        n >>= 7
    builder.append(n & 0xFF)

def _write_nibbles(builder: bytearray, lo: int, hi: int):
    builder.append((lo & 0xF) | ((hi & 0xF) << 4))

def _read_nibbles(reader: "_ByteReader") -> Tuple[int, int]:
    b = reader.next()
    return (b & 0xF, (b >> 4) & 0xF)


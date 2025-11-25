import chess
import chess.variant

from typing import Tuple, Optional, List, Union, Iterator
from itertools import zip_longest


class BinaryFen:
    """
    TODO
    """

    @staticmethod
    def read(data: bytes) -> chess.Board:
        """
        TODO
        """
        inner = bytearray(data)
        reader = iter(inner)
        occupied = _read_bitboard(reader)

        nibble_squares: List[Tuple[chess.Square, int]] = []
        iter_occupied = chess.scan_forward(occupied)
        for (sq1, sq2) in zip_longest(iter_occupied, iter_occupied):
            lo, hi = _read_nibbles(reader)
            nibble_squares.append((sq1, lo))
            if sq2 is not None:
                nibble_squares.append((sq2, hi))

        halfmove_clock = _read_leb128(reader)
        plies = _read_leb128(reader)

        variant = next(reader)
        board = _read_variant(variant)
        for sq, nibble in nibble_squares:
            _unpack_piece(board, sq, nibble)
        board.halfmove_clock = halfmove_clock
        # TODO TEST THAT
        board.fullmove_number = plies//2 + 1
        # from fullmove_number
        board.turn = plies % 2 == 0

        if isinstance(board, chess.variant.ThreeCheckBoard):
            lo, hi = _read_nibbles(reader)
            board.remaining_checks[chess.WHITE] = 3 - lo
            board.remaining_checks[chess.BLACK] = 3 - hi
        elif isinstance(board, chess.variant.CrazyhouseBoard):
            wp, bp = _read_nibbles(reader)
            wn, bn = _read_nibbles(reader)
            wb, bb = _read_nibbles(reader)
            wr, br = _read_nibbles(reader)
            wq, bq = _read_nibbles(reader)
            board.pockets[chess.WHITE] = _set_pocket(wp, wn, wb, wr, wq)
            board.pockets[chess.BLACK] = _set_pocket(bp, bn, bb, br, bq)
        return board



def _unpack_piece(board: chess.Board, sq: chess.Square, nibble: int):
    if nibble == 0:
        board.set_piece_at(sq, chess.Piece(chess.PAWN, chess.WHITE))
    elif nibble == 1:
        board.set_piece_at(sq, chess.Piece(chess.PAWN, chess.BLACK))
    elif nibble == 2:
        board.set_piece_at(sq, chess.Piece(chess.KNIGHT, chess.WHITE))
    elif nibble == 3:
        board.set_piece_at(sq, chess.Piece(chess.KNIGHT, chess.BLACK))
    elif nibble == 4:
        board.set_piece_at(sq, chess.Piece(chess.BISHOP, chess.WHITE))
    elif nibble == 5:
        board.set_piece_at(sq, chess.Piece(chess.BISHOP, chess.BLACK))
    elif nibble == 6:
        board.set_piece_at(sq, chess.Piece(chess.ROOK, chess.WHITE))
    elif nibble == 7:
        board.set_piece_at(sq, chess.Piece(chess.ROOK, chess.BLACK))
    elif nibble == 8:
        board.set_piece_at(sq, chess.Piece(chess.QUEEN, chess.WHITE))
    elif nibble == 9:
        board.set_piece_at(sq, chess.Piece(chess.QUEEN, chess.BLACK))
    elif nibble == 10:
        board.set_piece_at(sq, chess.Piece(chess.KING, chess.WHITE))
    elif nibble == 11:
        board.set_piece_at(sq, chess.Piece(chess.KING, chess.BLACK))
    elif nibble == 12:
        board.ep_square = sq ^ 8
        board.set_piece_at(sq, chess.Piece(chess.PAWN, chess.WHITE if chess.square_rank(sq) <= 4 else chess.BLACK))
    elif nibble == 13:
        board.castling_rights |= chess.BB_SQUARES[sq]
        board.set_piece_at(sq, chess.Piece(chess.ROOK, chess.WHITE))
    elif nibble == 14:
        board.castling_rights |= chess.BB_SQUARES[sq]
        board.set_piece_at(sq, chess.Piece(chess.ROOK, chess.BLACK))
    elif nibble == 15:
        board.turn = chess.BLACK
        board.set_piece_at(sq, chess.Piece(chess.KING, chess.BLACK)) 


def _read_bitboard(reader: Iterator[int]) -> chess.Bitboard:
    bb = chess.BB_EMPTY
    for _ in range(8):
        bb = (bb << 8) | (next(reader) & 0xFF)
    return bb

def _read_nibbles(reader: Iterator[int]) -> Tuple[int, int]:
    byte = next(reader)
    return byte & 0x0F, (byte >> 4) & 0x0F

def _write_nibbles(data: bytearray, lo: int, hi: int) -> None:
    data.append((hi << 4) | (lo & 0x0F)) 

def _read_leb128(reader: Iterator[int]) -> int:
    result = 0
    shift = 0
    while True:
        byte = next(reader)
        result |= (byte & 127) << shift
        if (byte & 128) == 0:
            break
        shift += 7
    # this is useless
    return result & 0x7fff_ffff

def _write_leb128(data: bytearray, value: int) -> None:
    while True:
        byte = value & 127
        value >>= 7
        if value != 0:
            byte |= 128
        data.append(byte)
        if value == 0:
            break

def _read_variant(byte: int) -> chess.Board:
    if byte == 1:
        return chess.variant.CrazyhouseBoard.empty()
    elif byte == 2:
        return chess.Board.empty(chess960=True)
    elif byte == 4:
        return chess.variant.KingOfTheHillBoard.empty()
    elif byte == 5:
        return chess.variant.ThreeCheckBoard.empty()
    elif byte == 6:
        return chess.variant.AntichessBoard.empty()
    elif byte == 7:
        return chess.variant.AtomicBoard.empty()
    elif byte == 8:
        return chess.variant.HordeBoard.empty()
    elif byte == 9:
        return chess.variant.RacingKingsBoard.empty()
    else: # 0 (std), 3 (from position) or fallback
        return chess.Board.empty()


# TODO, add this as method to ZH pocket?
# is this premature optimisation?
def _set_pocket(pawns: int, knights: int, bishops: int, rooks: int, queens: int) -> chess.variant.CrazyhousePocket:
    pocket = chess.variant.CrazyhousePocket()
    pocket._pieces = [-1, pawns, knights, bishops, rooks, queens]
    return pocket





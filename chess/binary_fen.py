from __future__ import annotations

# Almost all code based on: //github.com/lichess-org/scalachess/blob/8c94e2087f83affb9718fd2be19c34866c9a1a22/core/src/main/scala/format/BinaryFen.scala

from enum import IntEnum, verify, UNIQUE, CONTINUOUS

import logging

import chess
import chess.variant

from typing import Tuple, Optional, List, Union, Iterator, Literal
from dataclasses import dataclass
from itertools import zip_longest

LOGGER = logging.getLogger(__name__)

@verify(UNIQUE)
class StdMode(IntEnum):
    STANDARD = 0
    CHESS_960 = 2
    FROM_POSITION = 3

    @classmethod
    def from_int_opt(cls, value: int) -> Optional[StdMode]:
        """Convert an integer to a StdMode enum member, or return None if invalid."""
        try:
            return cls(value)
        except ValueError:
            return None

@verify(UNIQUE)
@verify(CONTINUOUS)
class VariantHeader(IntEnum):
    # chess/std
    STANDARD = 0
    CHESS_960 = 2
    FROM_POSITION = 3

    CRAZYHOUSE = 1
    KING_OF_THE_HILL = 4
    THREE_CHECK = 5
    ANTICHESS = 6
    ATOMIC = 7
    HORDE = 8
    RACING_KINGS = 9

    def board(self):
        if self == VariantHeader.CRAZYHOUSE:
            return chess.variant.CrazyhouseBoard.empty()
        elif self == VariantHeader.KING_OF_THE_HILL:
            return chess.variant.KingOfTheHillBoard.empty()
        elif self == VariantHeader.THREE_CHECK:
            return chess.variant.ThreeCheckBoard.empty()
        elif self == VariantHeader.ANTICHESS:
            return chess.variant.AntichessBoard.empty()
        elif self == VariantHeader.ATOMIC:
            return chess.variant.AtomicBoard.empty()
        elif self == VariantHeader.HORDE:
            return chess.variant.HordeBoard.empty()
        elif self == VariantHeader.RACING_KINGS:
            return chess.variant.RacingKingsBoard.empty()
        elif self in (VariantHeader.STANDARD, VariantHeader.CHESS_960, VariantHeader.FROM_POSITION):
            return chess.Board.empty(chess960=True)
        else:
            raise ValueError(f"Unsupported variant header: {self}")

CHESS_960_POSITIONS = [chess.Board.from_chess960_pos(i) for i in range(960)]


@dataclass(frozen=True)
class ThreeCheckData:
    white_received_checks: int
    black_received_checks: int

@dataclass(frozen=True)
class CrazyhouseData:
    white_pocket: chess.variant.CrazyhousePocket
    black_pocket: chess.variant.CrazyhousePocket
    promoted: chess.Bitboard


@dataclass(frozen=True)
class BinaryFen:
    """
    TODO
    """
    occupied: chess.Bitboard
    nibble_squares: List[Tuple[chess.Square, int]]
    halfmove_clock: int
    plies: int
    variant_header: int
    variant_data: Optional[Union[ThreeCheckData, CrazyhouseData]]

    @classmethod
    def parse(cls, data: bytes) -> BinaryFen:
        reader = iter(data)
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

        variant_header = next0(reader)

        variant_data = None
        if variant_header == VariantHeader.THREE_CHECK:
            lo, hi = _read_nibbles(reader)
            variant_data = ThreeCheckData(white_received_checks=lo, black_received_checks=hi)
        elif variant_header == VariantHeader.CRAZYHOUSE:
            wp, bp = _read_nibbles(reader)
            wn, bn = _read_nibbles(reader)
            wb, bb = _read_nibbles(reader)
            wr, br = _read_nibbles(reader)
            wq, bq = _read_nibbles(reader)
            # optimise?
            white_pocket = chess.variant.CrazyhousePocket("p"*wp + "n"*wn + "b"*wb + "r"*wr + "q"*wq)
            black_pocket = chess.variant.CrazyhousePocket("p"*bp + "n"*bn + "b"*bb + "r"*br + "q"*bq)
            promoted = _read_bitboard(reader)
            variant_data = CrazyhouseData(white_pocket=white_pocket, black_pocket=black_pocket, promoted=promoted)
        return cls(occupied=occupied,
                   nibble_squares=nibble_squares,
                   halfmove_clock=halfmove_clock,
                   plies=plies,
                   variant_header=variant_header,
                   variant_data=variant_data)

    @classmethod
    def decode(cls, data: bytes) -> Tuple[chess.Board, Optional[StdMode]]:
        """
        Read from bytes and return a chess.Board of the proper variant

        If it is standard chess position, also return the mode (standard, chess960, from_position)

        raise `ValueError` if data is invalid
        """
        binary_fen = cls.parse(data)

        variant = binary_fen.variant_header
        std_mode: Optional[StdMode] = StdMode.from_int_opt(binary_fen.variant_header)

        board = _read_variant(binary_fen.variant_header)
        for sq, nibble in binary_fen.nibble_squares:
            _unpack_piece(board, sq, nibble)
        board.halfmove_clock = binary_fen.halfmove_clock
        board.fullmove_number = binary_fen.plies//2 + 1
        # it is important to write it that way
        # because default turn can have been already set to black inside `_unpack_piece`
        if binary_fen.plies % 2 == 1:
            board.turn = chess.BLACK

        # TODO, use type(board).uci_variant instead? that would break typing
        if isinstance(board, chess.variant.ThreeCheckBoard) and isinstance(binary_fen.variant_data, ThreeCheckData):
            # remaining check are for the opposite side
            board.remaining_checks[chess.WHITE] = 3 - binary_fen.variant_data.black_received_checks
            board.remaining_checks[chess.BLACK] = 3 - binary_fen.variant_data.white_received_checks
        elif isinstance(board, chess.variant.CrazyhouseBoard) and isinstance(binary_fen.variant_data, CrazyhouseData):
            board.pockets[chess.WHITE] = binary_fen.variant_data.white_pocket
            board.pockets[chess.BLACK] = binary_fen.variant_data.black_pocket
            board.promoted = binary_fen.variant_data.promoted
        return (board, std_mode)


    @staticmethod
    def encode(board: chess.Board, std_mode: Optional[StdMode]=None) -> bytes:
        """
        Given a chess.Board, return its binary FEN representation, and std_mode if applicable

        If the board is a standard chess position, `std_mode` can be provided to specify the mode (standard, chess960, from_position)
        if not provided, it will be inferred from the root position
        """
        if std_mode is not None and type(board).uci_variant != "chess":
            raise ValueError("std_mode can only be provided for standard chess positions")
        builder = bytearray()
        _write_bitboard(builder, board.occupied)
        iter_occupied = chess.scan_forward(board.occupied)
        for (sq1, sq2) in zip_longest(iter_occupied, iter_occupied):
            lo = _pack_piece(board, sq1)
            hi = _pack_piece(board, sq2) if sq2 is not None else 0
            _write_nibbles(builder, lo, hi)
        plies = board.ply()
        broken_turn = board.king(chess.BLACK) is None and board.turn == chess.BLACK
        variant_header = std_mode if std_mode is not None else _encode_variant(board)
        if board.halfmove_clock > 0 or plies > 1 or broken_turn or variant_header != 0:
            _write_leb128(builder, board.halfmove_clock)

        if plies > 1 or broken_turn or variant_header != 0:
            _write_leb128(builder, plies)
        if variant_header != 0:
            builder.append(variant_header)
            if isinstance(board, chess.variant.ThreeCheckBoard):
                white_checks = 3 - board.remaining_checks[chess.WHITE]
                black_checks = 3 - board.remaining_checks[chess.BLACK]
                _write_nibbles(builder, black_checks, white_checks)
            elif isinstance(board, chess.variant.CrazyhouseBoard):
                for piece_type in chess.PIECE_TYPES[:-1]:
                    [w, b] = [board.pockets[color].count(piece_type) for color in chess.COLORS]
                    _write_nibbles(builder, w, b)
                if board.promoted:
                    _write_bitboard(builder, board.promoted)
        return bytes(builder)

def _encode_variant(board: chess.Board) -> int:
    uci_variant = type(board).uci_variant
    if uci_variant == "chess":
        root = board.root()
        if root in CHESS_960_POSITIONS:
            return 2 # chess960
        # TODO FIXME check it works properly
        elif root == chess.Board():
            return 0
        else:
            return 3
    elif uci_variant == "crazyhouse":
        return 1
    elif uci_variant == "kingofthehill":
        return 4
    elif uci_variant == "3check":
        return 5
    elif uci_variant == "antichess":
        return 6
    elif uci_variant == "atomic":
        return 7
    elif uci_variant == "horde":
        return 8
    elif uci_variant == "racingkings":
        return 9
    else:
        raise ValueError(f"Unsupported variant: {uci_variant}")


def _pack_piece(board: chess.Board, sq: chess.Square) -> int:
    # Encoding from
    # https://github.com/official-stockfish/nnue-pytorch/blob/2db3787d2e36f7142ea4d0e307b502dda4095cd9/lib/nnue_training_data_formats.h#L4607
    piece = board.piece_at(sq)
    if piece is None:
        raise ValueError(f"Unreachable: no piece at square {sq}, board: {board}")
    if piece.piece_type == chess.PAWN:
        if board.ep_square is not None and (board.ep_square + 8 == sq or board.ep_square - 8 == sq):
            return 12
        return 0 if piece.color == chess.WHITE else 1
    elif piece.piece_type == chess.KNIGHT:
        return 2 if piece.color == chess.WHITE else 3
    elif piece.piece_type == chess.BISHOP:
        return 4 if piece.color == chess.WHITE else 5
    elif piece.piece_type == chess.ROOK:
        if board.castling_rights & chess.BB_SQUARES[sq]:
            return 13 if piece.color == chess.WHITE else 14
        return 6 if piece.color == chess.WHITE else 7
    elif piece.piece_type == chess.QUEEN:
        return 8 if piece.color == chess.WHITE else 9
    elif piece.piece_type == chess.KING:
        if piece.color == chess.BLACK and board.turn == chess.BLACK:
            return 15
        return 10 if piece.color == chess.WHITE else 11
    raise ValueError(f"Unreachable: unknown piece {piece} at square {sq}, board: {board}")

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
        # in scalachess rank starts at 1, python-chess 0
        rank = chess.square_rank(sq)
        if rank == 3:
            color = chess.WHITE
        elif rank == 4:
            color = chess.BLACK
        else:
            raise ValueError(f"Pawn at square {chess.square_name(sq)} cannot be an en passant pawn")
        board.ep_square = sq - 8 if color else sq + 8
        board.set_piece_at(sq, chess.Piece(chess.PAWN, color))
    elif nibble == 13:
        board.castling_rights |= chess.BB_SQUARES[sq]
        board.set_piece_at(sq, chess.Piece(chess.ROOK, chess.WHITE))
    elif nibble == 14:
        board.castling_rights |= chess.BB_SQUARES[sq]
        board.set_piece_at(sq, chess.Piece(chess.ROOK, chess.BLACK))
    elif nibble == 15:
        board.turn = chess.BLACK
        board.set_piece_at(sq, chess.Piece(chess.KING, chess.BLACK)) 

def next0(reader: Iterator[int]) -> int:
    return next(reader, 0)

def _read_bitboard(reader: Iterator[int]) -> chess.Bitboard:
    bb = chess.BB_EMPTY
    for _ in range(8):
        bb = (bb << 8) | (next0(reader) & 0xFF)
    return bb

def _write_bitboard(data: bytearray, bb: chess.Bitboard) -> None:
    for shift in range(56, -1, -8):
        data.append((bb >> shift) & 0xFF)

def _read_nibbles(reader: Iterator[int]) -> Tuple[int, int]:
    byte = next0(reader)
    return byte & 0x0F, (byte >> 4) & 0x0F

def _write_nibbles(data: bytearray, lo: int, hi: int) -> None:
    data.append((hi << 4) | (lo & 0x0F)) 

def _read_leb128(reader: Iterator[int]) -> int:
    result = 0
    shift = 0
    while True:
        byte = next0(reader)
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
    elif byte == 0 or byte == 2 or byte == 3: # 0 (std), 2 (chess960) 3 (from position)
        return chess.Board.empty(chess960=True)
    # TODO, is this better than silently trying std?
    else:
        raise ValueError(f"Unsupported variant byte: {byte}")





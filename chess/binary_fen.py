import chess

from typing import Tuple, Optional, List, Union
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
        inner = bytesarray(data)
        reader = iter(inner)
        occupied = _read_bitboard(reader)

        nibble_squares: List[(chess.Square, int)] = []
        iter_occupied = chess.scan_forward(occupied)
        for (sq1, sq2) in zip_longest(iter_occupied, iter_occupied):
            lo, hi = _read_nibbles(reader)
            nibble_squares.append((sq1, lo))
            if sq2 is not None:
                nibble_squares.append((sq2, hi))

        halfmove_clock = _read_leb128(reader)
        fullmove_number = _read_leb128(reader)

        board = 
        

    def unpack_piece(board: chess.Board, sq: chess.Square, nibble: int):
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
            board.set_piece_at(sq, chess.Piece(chess.PAWN, chess.WHITE if sq.rank <= 4 else chess.BLACK))
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

    def _read_nibbles(reader: Iterator[int]) -> (int, int):
        byte = next(reader)
        return byte & 0x0F, (byte >> 4) & 0x0F

    def _read_leb128(reader: Iterator[int]) -> int:
        result = 0
        shift = 0
        while True:
            byte = next(reader)
            result |= (byte & 127) << shift
            if (byte & 128) == 0:
                break
            shift += 7
        return result & 0x7fff_ffff

    def _read_variant(reader: Iterator[int]) -> chess.Board:
        byte = next(reader)
      #   reader.next match
      # case 0 => Standard
      # case 1 => Crazyhouse
      # case 2 => Chess960
      # case 3 => FromPosition
      # case 4 => KingOfTheHill
      # case 5 => ThreeCheck
      # case 6 => Antichess
      # case 7 => Atomic
      # case 8 => Horde
      # case 9 => RacingKings
      # case _ => Standard
        if byte == 0:
            return chess.Board()
        elif byte == 1:
            return chess.VariantCrazyhouse()
        elif byte == 2:
            return chess.VariantChess960()
        elif byte == 3:
            return chess.VariantFromPosition()
        elif byte == 4:
            return chess.VariantKingOfTheHill()
        elif byte == 5:
            return chess.VariantThreeCheck()
        elif byte == 6:
            return chess.VariantAntichess()
        elif byte == 7:
            return chess.VariantAtomic()
        elif byte == 8:
            return chess.VariantHorde()
        elif byte == 9:
            return chess.VariantRacingKings()
        else:
            return chess.VariantStandard()







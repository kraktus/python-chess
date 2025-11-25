import chess
import struct
from typing import Tuple, Optional, List, Union


class BinaryFenError(Exception):
    """Custom exception for BinaryFen errors."""
    pass


class BinaryFen:
    """
    A class to serialize and deserialize chess positions (including variants)
    to and from a compact binary format.

    Supports all variants handled in the Scala implementation:
    Standard, Chess960, FromPosition, KingOfTheHill, ThreeCheck, Antichess,
    Atomic, Horde, RacingKings, Crazyhouse.
    """

    VARIANT_MAP = {
        chess.Board: 0,
        chess.Chess960: 2,
        chess.FromPosition: 3,
        chess.KingOfTheHill: 4,
        chess.ThreeCheck: 5,
        chess.Antichess: 6,
        chess.Atomic: 7,
        chess.Horde: 8,
        chess.RacingKings: 9,
        chess.Crazyhouse: 1,
    }

    @classmethod
    def write(cls, board: chess.Board) -> bytes:
        """
        Serializes a chess.Board (or variant subclass) to binary FEN.

        Args:
            board (chess.Board): The board to serialize.

        Returns:
            bytes: The serialized binary FEN.
        """
        builder = bytearray()

        # Serialize variant
        variant_id = cls.VARIANT_MAP.get(type(board), 0)
        builder.append(variant_id)

        # Serialize board position
        builder.extend(cls._serialize_board(board))

        # Serialize variant-specific data
        if isinstance(board, chess.Crazyhouse):
            builder.extend(cls._serialize_crazyhouse(board))
        elif isinstance(board, chess.ThreeCheck):
            builder.extend(cls._serialize_threecheck(board))

        return bytes(builder)

    @classmethod
    def read(cls, data: bytes) -> chess.Board:
        """
        Deserializes binary FEN to a chess.Board (or appropriate variant subclass).

        Args:
            data (bytes): The binary FEN data.

        Returns:
            chess.Board: The deserialized board.
        """
        reader = memoryview(data)

        # Deserialize variant
        variant_id = reader[0]
        variant_class = cls._get_variant_class(variant_id)

        # Deserialize board position
        board = cls._deserialize_board(reader[1:], variant_class)

        # Deserialize variant-specific data
        if variant_class == chess.Crazyhouse:
            cls._deserialize_crazyhouse(reader[1:], board)
        elif variant_class == chess.ThreeCheck:
            cls._deserialize_threecheck(reader[1:], board)

        return board

    @staticmethod
    def _serialize_board(board: chess.Board) -> bytes:
        """
        Serializes the board position to binary.

        Args:
            board (chess.Board): The board to serialize.

        Returns:
            bytes: The serialized board position.
        """
        # Example: Serialize board FEN and turn
        return struct.pack("64s?", board.board_fen().encode(), board.turn)

    @staticmethod
    def _deserialize_board(data: memoryview, variant_class: type) -> chess.Board:
        """
        Deserializes the board position from binary.

        Args:
            data (memoryview): The binary data.
            variant_class (type): The class of the variant.

        Returns:
            chess.Board: The deserialized board.
        """
        board_fen, turn = struct.unpack("64s?", data[:65])
        board = variant_class(board_fen.decode())
        board.turn = turn
        return board

    @staticmethod
    def _serialize_crazyhouse(board: chess.Crazyhouse) -> bytes:
        """
        Serializes Crazyhouse-specific data.

        Args:
            board (chess.Crazyhouse): The Crazyhouse board.

        Returns:
            bytes: The serialized Crazyhouse data.
        """
        pockets = board.pockets
        return struct.pack("16B", *pockets.white, *pockets.black)

    @staticmethod
    def _deserialize_crazyhouse(data: memoryview, board: chess.Crazyhouse) -> None:
        """
        Deserializes Crazyhouse-specific data.

        Args:
            data (memoryview): The binary data.
            board (chess.Crazyhouse): The Crazyhouse board to update.
        """
        pockets = struct.unpack("16B", data[:16])
        board.pockets.white = pockets[:8]
        board.pockets.black = pockets[8:]

    @staticmethod
    def _serialize_threecheck(board: chess.ThreeCheck) -> bytes:
        """
        Serializes ThreeCheck-specific data.

        Args:
            board (chess.ThreeCheck): The ThreeCheck board.

        Returns:
            bytes: The serialized ThreeCheck data.
        """
        return struct.pack("2B", board.checks_white, board.checks_black)

    @staticmethod
    def _deserialize_threecheck(data: memoryview, board: chess.ThreeCheck) -> None:
        """
        Deserializes ThreeCheck-specific data.

        Args:
            data (memoryview): The binary data.
            board (chess.ThreeCheck): The ThreeCheck board to update.
        """
        checks_white, checks_black = struct.unpack("2B", data[:2])
        board.checks_white = checks_white
        board.checks_black = checks_black

    @classmethod
    def _get_variant_class(cls, variant_id: int) -> type:
        """
        Maps a variant ID to the corresponding chess.Board subclass.

        Args:
            variant_id (int): The variant ID.

        Returns:
            type: The chess.Board subclass.
        """
        for variant_class, vid in cls.VARIANT_MAP.items():
            if vid == variant_id:
                return variant_class
        raise BinaryFenError(f"Unknown variant ID: {variant_id}")

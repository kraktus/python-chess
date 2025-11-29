#!/usr/bin/env python3

import asyncio
import copy
import logging
import os
import os.path
import platform
import sys
import tempfile
import textwrap
import unittest
import io

import chess
import chess.variant
import chess.binary_fen

from chess.binary_fen import BinaryFen

KOTH = chess.variant.KingOfTheHillBoard
THREE_CHECKS = chess.variant.ThreeCheckBoard
ANTI = chess.variant.AntichessBoard
ATOMIC = chess.variant.AtomicBoard
HORDE =  chess.variant.HordeBoard
RK = chess.variant.RacingKingsBoard
ZH = chess.variant.CrazyhouseBoard

class BinaryFenTestCase(unittest.TestCase):

    def test_nibble_roundtrip(self):
        for lo in range(16):
            for hi in range(16):
                data = bytearray()
                chess.binary_fen._write_nibbles(data, lo, hi)
                read_lo, read_hi = chess.binary_fen._read_nibbles(iter(data))
                self.assertEqual(lo, read_lo)
                self.assertEqual(hi, read_hi)

    def test_bitboard_roundtrip(self):
        test_bitboards = [
            0x0000000000000000,
            0xFFFFFFFFFFFFFFFF,
            0x1234567890ABCDEF,
            0x0F0F0F0F0F0F0F0F,
            0xF0F0F0F0F0F0F0F0,
            0x8000000000000001,
            0x7FFFFFFFFFFFFFFE,
        ]
        for bb in test_bitboards:
            data = bytearray()
            chess.binary_fen._write_bitboard(data, bb)
            read_bb = chess.binary_fen._read_bitboard(iter(data))
            self.assertEqual(bb, read_bb)

    def test_leb128_roundtrip(self):
        test_values = [
            0,
            1,
            127,
            128,
            255,
            16384,
            2097151,
            268435455,
            2147483647,
        ]
        for value in test_values:
            data = bytearray()
            chess.binary_fen._write_leb128(data, value)
            read_value = chess.binary_fen._read_leb128(iter(data))
            self.assertEqual(value, read_value)


    def test_read_binary_fen_std(self):
        test_cases = [
            ("0000000000000000", "8/8/8/8/8/8/8/8 w - - 0 1"),
            ("00000000000000000001", "8/8/8/8/8/8/8/8 b - - 0 1"),
            ("000000000000000064df06", "8/8/8/8/8/8/8/8 b - - 100 432"),
            ("ffff00001000efff2d844ad200000000111111113e955fe3", "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1"),
            ("20400006400000080ac0b1", "5k2/6p1/8/1Pp5/6P1/8/8/3K4 w - c6 0 1"),
            ("10000000180040802ac10f", "4k3/8/8/8/3pP3/8/6N1/7K b - e3 0 1"),
            # this is encoded with `standard` variant but with chess960 castling
            # should this be accepted? for now basing on scalachess behavior
            ("8901080000810091ad0d10e1f70007", "r2r3k/p7/3p4/8/8/P6P/8/R3K2R b KQq - 0 4"),
            ("00000002180000308a1c0f030103", "8/8/8/1k6/3Pp3/8/8/4KQ2 b - d3 3 1")
        ]
        for binary_fen, expected_fen in test_cases:
            with self.subTest(binary_fen=binary_fen, expected_fen=expected_fen):
                self.check_binary(binary_fen, expected_fen)


    # for python-chess, 960 is handled the same as std
    def test_read_binary_fen_960(self):
        test_cases = [("704f1ee8e81e4f70d60a44000002020813191113511571be000402", "4rrk1/pbbp2p1/1ppnp3/3n1pqp/3N1PQP/1PPNP3/PBBP2P1/4RRK1 w Ff - 0 3")]
        for binary_fen, expected_fen in test_cases:
            with self.subTest(binary_fen=binary_fen, expected_fen=expected_fen):
                self.check_binary(binary_fen, expected_fen)

    def test_read_binary_fen_variants(self):
        test_cases = [("ffff00000000ffff2d844ad200000000111111113e955be3000004", "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", KOTH),
        #("ffff00000000ffff2d844ad200000000111111113e955be363000501", "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 99 1 +0+1", THREE_CHECKS),
        ("00800000000008001a000106", "8/7p/8/8/8/8/3K4/8 b - - 0 1", ANTI),
        ("ffff00000000ffff2d844ad200000000111111113e955be3020407", "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 2 3", ATOMIC),
        ("ffff0066ffffffff000000000000000000000000000000000000111111113e955be3000008", "rnbqkbnr/pppppppp/8/1PP2PP1/PPPPPPPP/PPPPPPPP/PPPPPPPP/PPPPPPPP w kq - 0 1", HORDE),
        ("000000000000ffff793542867b3542a6000009", "8/8/8/8/8/8/krbnNBRK/qrbnNBRQ w - - 0 1", RK),
        ("ffff00000000ffff2d844ad200000000111111113e955be300e407010000000000ef0000000000002a", "r~n~b~q~kb~n~r~/pppppppp/8/8/8/8/PPPPPPPP/RN~BQ~KB~NR/ w KQkq - 0 499", ZH),
        ("ffff00000000ffff2d844ad200000000111111113e955be30000010000000000", "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR/ w KQkq - 0 1", ZH)
        ]
        for binary_fen, expected_fen, variant in test_cases:
            with self.subTest(binary_fen=binary_fen, expected_fen=expected_fen, variant=variant):
                self.check_binary(binary_fen, expected_fen, variant)


    def check_binary(self, binary_fen, expected_fen, variant = None):
        compressed = bytes.fromhex(binary_fen)
        board = BinaryFen.decode(compressed)
        encoded = BinaryFen.encode(board)
        from_fen = chess.Board(fen=expected_fen, chess960=True) if variant is None else variant(fen=expected_fen)
        self.assertEqual(board, from_fen)
        self.assertEqual(board.fen(), BinaryFen.decode(encoded).fen())
        self.assertEqual(encoded, compressed)
        # different FEN format exist for these variants
        if variant not in [ZH, THREE_CHECKS]:
            self.assertEqual(expected_fen, board.fen())


if __name__ == "__main__":
    unittest.main()
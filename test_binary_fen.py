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
import chess.binary_fen.BinaryFen

class NibbleTest(unittest.TestCase):

    def test_nibble_roundtrip(self):
        for lo in range(128):
            for hi in range(128):
                data = bytearray()
                BinaryFen._write_nibbles(data, lo, hi)
                read_lo, read_hi = chess.binary_fen._read_nibbles(iter(data))
                self.assertEqual(lo, read_lo)
                self.assertEqual(hi, read_hi)

class BinaryFenTest(unittest.TestCase):
    pass


if __name__ == "__main__":
    unittest.main()
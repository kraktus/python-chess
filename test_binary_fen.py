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

class BinaryFenTest(unittest.TestCase):
    